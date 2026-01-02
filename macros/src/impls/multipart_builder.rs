//! Generates `MultipartBuilder` impl for incremental multipart field accumulation.
//!
//! For a struct `Foo`, generates:
//! - `FooMultipartBuilder` with fields wrapped for accumulation:
//!   - `T` → `Option<T>` (track presence)
//!   - `Option<T>` / `Vec<T>` (non-nested) → kept as-is
//!   - `#[form_data(nested)] T` → `TMultipartBuilder`
//!   - `#[form_data(nested)] Option<T>` → `Option<TMultipartBuilder>`
//!   - `#[form_data(nested)] Vec<T>` → `BTreeMap<usize, TMultipartBuilder>` (sparse indices)
//!
//! - `impl MultipartBuilder<S>` with:
//!   - `consume()`: Routes fields by name segment matching. Nested fields use prefix
//!     matching and delegate to inner builders with adjusted span. Leaf fields use
//!     exact matching. Uses `Spanned<&str>` to track current segment within full name.
//!   - `finalize()`: Builds target struct, applying defaults or returning `MissingField`.

use crate::case_conversion::RenameCase;
use crate::derive_input::{FieldData, InputData};
use crate::util::{
    extract_inner_type, matches_option_signature, matches_vec_signature, strip_leading_rawlit,
};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::quote;
use std::collections::BTreeMap;

#[allow(dead_code)] // for now this macro is derived together with TryFromMultipart
pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match InputData::from_derive_input(&input) {
        Ok(input) => expand(input).into(),
        Err(err) => err.write_errors().into(),
    }
}

pub fn expand(input: InputData) -> proc_macro2::TokenStream {
    let input_generic = input.generic();
    let input_state_ty = input.state_ty();
    let input_builder_ident = input.builder_ident();

    let InputData { ident, vis, data, strict, rename_all, .. } = input;
    let fields = data.take_struct().unwrap();

    // Compute field names once, map name → field (BTreeMap for deterministic iteration order)
    let fields: BTreeMap<_, _> = fields.iter().map(|f| (form_name(f, rename_all), f)).collect();

    let struct_def = {
        let builder_fields = fields.values().map(|FieldData { ident, ty, nested, .. }| {
            let ty = if *nested {
                nested_builder_type(ty)
            } else if matches_vec_signature(ty) || matches_option_signature(ty) {
                quote! { #ty }
            } else {
                quote! { std::option::Option<#ty> }
            };
            quote! { #ident: #ty }
        });
        quote! {
            #[doc(hidden)]
            #[derive(Default)]
            #vis struct #input_builder_ident {
                #(#builder_fields),*
            }
        }
    };

    let impl_block = {
        let consume_method = {
            let branches = fields.iter().map(|(name, FieldData { ident, ty, limit, nested, .. })| {
                let prefixed_name = format!(".{}", name);
                let prefix_len = prefixed_name.len();
                if *nested {
                    quote! {
                        if __segment__.starts_with(&#prefixed_name[__offset__..]) {
                            let __new_start__ = __span__.start + #prefix_len - __offset__;
                            let __new_name__ = axum_typed_multipart::Spanned::new(
                                __new_start__..(__span__.end),
                                __full__,
                            );
                            __field__ = match self
                                .#ident
                                .consume(__field__, __new_name__, __state__, __depth__ + 1)
                                .await?
                            {
                                Some(__f__) => __f__,
                                None => return Ok(None),
                            };
                        }
                    }
                } else {
                    let value = quote! {
                        <_ as axum_typed_multipart::TryFromFieldWithState<_>>::try_from_field_with_state(__field__, #limit, __state__).await?
                    };

                    let assignment = if matches_vec_signature(ty) {
                        quote! { self.#ident.push(#value); }
                    } else {
                        let assignment = quote! { self.#ident = Some(#value); };
                        if !strict {
                            assignment
                        } else {
                            quote! {
                                if self.#ident.is_none() {
                                    #assignment
                                } else {
                                    return Err(axum_typed_multipart::TypedMultipartError::DuplicateField {
                                        field_name: __field__.name().unwrap_or_default().to_string()
                                    });
                                }
                            }
                        }
                    };

                    quote! {
                        if __segment__ == &#prefixed_name[__offset__..] {
                            #assignment
                            return Ok(None);
                        }
                    }
                }
            });

            quote! {
                async fn consume<'a>(
                    &mut self,
                    mut __field__: axum::extract::multipart::Field<'a>,
                    __name__: axum_typed_multipart::Spanned<&str>,
                    __state__: &#input_state_ty,
                    __depth__: usize,
                ) -> Result<Option<axum::extract::multipart::Field<'a>>, axum_typed_multipart::TypedMultipartError> {
                    let __full__ = *__name__.as_ref();
                    let __span__ = __name__.span();
                    let __segment__ = &__full__[__span__.start..__span__.end];
                    // At depth 0, no leading dot expected; at depth > 0, skip the leading dot in prefixed names
                    let __offset__ = if __depth__ == 0 { 1 } else { 0 };
                    #(#branches)*
                    Ok(Some(__field__))
                }
            }
        };

        let finalize_method = {
            let field_assignments =
                fields.iter().map(|(name, FieldData { ident, ty, default, nested, .. })| {
                    let field_path = quote! {
                        if __path__.is_empty() { #name.to_string() } else { format!("{}.{}", __path__, #name) }
                    };
                    let value = if *nested {
                        let finalize = quote! {
                            axum_typed_multipart::MultipartBuilder::<#input_state_ty>::finalize(
                                self.#ident,
                                &#field_path
                            )
                        };
                        if *default {
                            quote! { #finalize.unwrap_or_default() }
                        } else {
                            quote! { #finalize? }
                        }
                    } else if matches_vec_signature(ty) || matches_option_signature(ty) {
                        quote! { self.#ident }
                    } else if *default {
                        quote! { self.#ident.unwrap_or_default() }
                    } else {
                        quote! {
                            self.#ident.ok_or_else(||
                                axum_typed_multipart::TypedMultipartError::MissingField {
                                    field_name: #field_path
                                }
                            )?
                        }
                    };
                    quote! {
                        #ident: #value
                    }
                });

            quote! {
                fn finalize(self, __path__: &str) -> Result<Self::Target, axum_typed_multipart::TypedMultipartError> {
                    Ok(Self::Target { #(#field_assignments),* })
                }
            }
        };

        quote! {
            #[axum_typed_multipart::async_trait]
            impl #input_generic axum_typed_multipart::MultipartBuilder<#input_state_ty> for #input_builder_ident {
                type Target = #ident;
                #consume_method
                #finalize_method
            }
        }
    };

    quote! {
        #struct_def
        #impl_block
    }
}

/// Generates builder ident: `Foo` → `FooMultipartBuilder`
pub fn builder_ident(ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("{ident}MultipartBuilder"), ident.span())
}

/// Converts type path to builder path: `foo::Bar` → `foo::BarMultipartBuilder`
fn to_builder_type(mut ty: syn::Type) -> syn::Type {
    let syn::Type::Path(ref mut tp) = ty else {
        abort!(ty, "nested field must be a simple type path");
    };
    if let Some(last) = tp.path.segments.last_mut() {
        last.ident = builder_ident(&last.ident);
        ty
    } else {
        abort!(tp, "nested field type path cannot be empty");
    }
}

// Computes field name with optional case conversion.
fn form_name(field: &FieldData, rename_all: Option<RenameCase>) -> String {
    if let Some(name) = &field.field_name {
        return name.to_string();
    }
    let ident = field.ident.as_ref().unwrap().to_string();
    let name = strip_leading_rawlit(&ident);
    match rename_all {
        Some(case) => case.convert_case(&name),
        None => name,
    }
}

fn nested_builder_type(ty: &syn::Type) -> proc_macro2::TokenStream {
    if matches_option_signature(ty) {
        let inner = extract_inner_type(ty);
        let inner_builder = nested_builder_type(inner);
        quote! { Option<#inner_builder> }
    } else if matches_vec_signature(ty) {
        let inner = extract_inner_type(ty);
        let inner_builder = nested_builder_type(inner);
        quote! { std::collections::BTreeMap<usize, #inner_builder> }
    } else {
        let ty = to_builder_type(ty.clone());
        quote! { #ty }
    }
}
