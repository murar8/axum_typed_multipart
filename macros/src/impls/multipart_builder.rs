use crate::case_conversion::RenameCase;
use crate::derive_input::{FieldData, InputData};
use crate::limit_bytes::LimitBytes;
use crate::util::{
    extract_inner_type, matches_option_signature, matches_vec_signature, strip_leading_rawlit,
};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeMap;

pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match InputData::from_derive_input(&input) {
        Ok(input) => gen::output(input).into(),
        Err(err) => return err.write_errors().into(),
    }
}

/// Generates `FooMultipartBuilder` struct and its `MultipartBuilder` impl.
pub mod gen {
    use super::*;

    // Generates builder ident: `Foo` → `FooMultipartBuilder`
    pub fn builder_ident(ty: &impl quote::ToTokens) -> syn::Ident {
        use syn::spanned::Spanned;
        syn::Ident::new(&format!("{}MultipartBuilder", ty.to_token_stream()), ty.span())
    }

    // Computes field name with optional case conversion.
    fn field_name(field: &FieldData, rename_all: Option<RenameCase>) -> String {
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

    pub fn output(input: InputData) -> proc_macro2::TokenStream {
        let input_generic = input.generic();
        let input_state_ty = input.state_ty();
        let input_builder_ident = input.builder_ident();

        let InputData { ident, data, strict, rename_all, .. } = input;
        let fields = data.take_struct().unwrap();

        // Compute field names once, map name → field (BTreeMap for deterministic iteration order)
        let fields: BTreeMap<_, _> =
            fields.iter().map(|f| (field_name(f, rename_all), f)).collect();

        let struct_def = struct_def(&input_builder_ident, &fields);
        let impl_block = impl_block(
            &ident,
            &input_generic,
            &input_state_ty,
            &input_builder_ident,
            &fields,
            strict,
        );

        quote! {
            #struct_def
            #impl_block
        }
    }

    /// Generates: `#[derive(Default)] struct FooMultipartBuilder { .. }`
    fn struct_def(
        input_builder_ident: &syn::Ident,
        fields: &BTreeMap<String, &FieldData>,
    ) -> proc_macro2::TokenStream {
        let builder_fields = fields.values().map(|f| struct_def::field(f));
        quote! {
            #[derive(Default)]
            struct #input_builder_ident {
                #(#builder_fields),*
            }
        }
    }

    /// Helpers for generating builder struct fields.
    mod struct_def {
        use super::*;

        /// Dispatches to simple or nested field generation.
        pub fn field(FieldData { ident, ty, nested, .. }: &FieldData) -> proc_macro2::TokenStream {
            let ty = if *nested { field_ty::nested(ty) } else { field_ty::simple(ty) };
            quote! { #ident: #ty }
        }

        mod field_ty {
            use super::*;

            /// Returns builder field type for simple types.
            /// Example: `String` → `Option<String>`
            /// Example: `Vec<String>` → `Vec<String>`
            pub fn simple(ty: &syn::Type) -> proc_macro2::TokenStream {
                if matches_vec_signature(ty) || matches_option_signature(ty) {
                    quote! { #ty }
                } else {
                    quote! { std::option::Option<#ty> }
                }
            }

            /// Returns builder field type for nested structs (recursive for container types).
            /// Example: `Address` → `AddressMultipartBuilder`
            /// Example: `Vec<Address>` → `BTreeMap<usize, AddressMultipartBuilder>`
            /// Example: `Option<Address>` → `Option<AddressMultipartBuilder>`
            /// Example: `Option<Vec<Address>>` → `Option<BTreeMap<usize, AddressMultipartBuilder>>`
            /// Example: `Vec<Option<Address>>` → `BTreeMap<usize, Option<AddressMultipartBuilder>>`
            pub fn nested(ty: &syn::Type) -> proc_macro2::TokenStream {
                if matches_option_signature(ty) {
                    let inner = extract_inner_type(ty);
                    let inner_builder = nested(inner); // recurse
                    quote! { Option<#inner_builder> }
                } else if matches_vec_signature(ty) {
                    let inner = extract_inner_type(ty);
                    let inner_builder = nested(inner); // recurse
                    quote! { std::collections::BTreeMap<usize, #inner_builder> }
                } else {
                    let field_builder_ident = builder_ident(ty);
                    quote! { #field_builder_ident }
                }
            }
        }
    }

    /// Generates: `impl MultipartBuilder<S> for FooMultipartBuilder { .. }`
    fn impl_block(
        ident: &syn::Ident,
        input_generic: &impl quote::ToTokens,
        input_state_ty: &impl quote::ToTokens,
        input_builder_ident: &syn::Ident,
        fields: &BTreeMap<String, &FieldData>,
        strict: bool,
    ) -> proc_macro2::TokenStream {
        let consume_method = impl_block::consume(input_state_ty, fields, strict);
        let finalize_method = impl_block::finalize(ident, fields, input_state_ty);
        quote! {
            #[axum_typed_multipart::async_trait]
            impl #input_generic axum_typed_multipart::MultipartBuilder<#input_state_ty> for #input_builder_ident {
                type Target = #ident;
                #consume_method
                #finalize_method
            }
        }
    }

    /// Helpers for generating MultipartBuilder impl methods.
    mod impl_block {
        use super::*;

        /// Generates: `async fn consume(&mut self, field, name, state) -> Result<Option<Field>, _> { .. }`
        pub fn consume(
            input_state_ty: &impl quote::ToTokens,
            fields: &BTreeMap<String, &FieldData>,
            strict: bool,
        ) -> proc_macro2::TokenStream {
            let branches = fields.iter().map(|(name, field)| consume::branch(name, field, strict));

            quote! {
                async fn consume<'a>(
                    &mut self,
                    mut __field__: axum::extract::multipart::Field<'a>,
                    __name__: axum_typed_multipart::Spanned<&str>,
                    __state__: &#input_state_ty,
                ) -> Result<Option<axum::extract::multipart::Field<'a>>, axum_typed_multipart::TypedMultipartError> {
                    let __full__ = *__name__.as_ref();
                    let __span__ = __name__.span();
                    let __segment__ = &__full__[__span__.start..__span__.end];
                    #(#branches)*
                    Ok(Some(__field__))
                }
            }
        }

        /// Helpers for generating consume method branches.
        ///
        /// Field names use a dot-prefix convention to prevent prefix collisions. For example,
        /// with fields `user` (nested) and `username` (simple), incoming `username` becomes
        /// `.username`. The nested `user` field strips `.user` via `strip_prefix`, leaving
        /// `name` (no dot). The inner builder then checks for `.name`, which doesn't match
        /// `name`, so the field falls through to correctly match the simple `.username` field.
        mod consume {
            use super::*;

            /// Dispatches to simple or nested branch generation.
            pub fn branch(name: &str, field: &FieldData, strict: bool) -> proc_macro2::TokenStream {
                if field.nested {
                    branch::nested(name, field)
                } else {
                    branch::simple(name, field, strict)
                }
            }

            mod branch {
                use super::*;

                /// Generates match branch for simple field consumption.
                /// Example: `if __name__ == ".name" { self.name = Some(value); return Ok(None); }`
                pub fn simple(
                    name: &str,
                    FieldData { ident, ty, limit, .. }: &FieldData,
                    strict: bool,
                ) -> proc_macro2::TokenStream {
                    let prefixed_name = format!(".{}", name);
                    let limit_bytes = limit.unwrap_or(LimitBytes(None));

                    let value = quote! {
                        <_ as axum_typed_multipart::TryFromFieldWithState<_>>::try_from_field_with_state(__field__, #limit_bytes, __state__).await?
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
                        if __segment__ == #prefixed_name {
                            #assignment
                            return Ok(None);
                        }
                    }
                }

                /// Generates match branch that delegates to nested builder.
                /// Example: `if __segment__.starts_with(".addr") { __field__ = match self.addr.consume(.., new_spanned).await? { .. } }`
                pub fn nested(
                    name: &str,
                    FieldData { ident, .. }: &FieldData,
                ) -> proc_macro2::TokenStream {
                    let prefixed_name = format!(".{}", name);
                    let prefix_len = prefixed_name.len();
                    quote! {
                        if __segment__.starts_with(#prefixed_name) {
                            let __new_name__ = axum_typed_multipart::Spanned::new(__span__.start + #prefix_len..__span__.end, __full__);
                            __field__ = match self.#ident.consume(__field__, __new_name__, __state__).await? {
                                Some(__f__) => __f__,
                                None => return Ok(None),
                            };
                        }
                    }
                }
            }
        }

        /// Generates: `fn finalize(self) -> Result<Foo, _> { Ok(Foo { .. }) }`
        pub fn finalize(
            ident: &syn::Ident,
            fields: &BTreeMap<String, &FieldData>,
            input_state_ty: &impl quote::ToTokens,
        ) -> proc_macro2::TokenStream {
            let field_assignments =
                fields.iter().map(|(name, field)| finalize::field(name, field, input_state_ty));

            quote! {
                fn finalize(self) -> Result<Self::Target, axum_typed_multipart::TypedMultipartError> {
                    Ok(#ident { #(#field_assignments),* })
                }
            }
        }

        /// Helpers for generating finalize method field assignments.
        mod finalize {
            use super::*;

            /// Dispatches to simple or nested field assignment.
            pub fn field(
                name: &str,
                field: &FieldData,
                input_state_ty: &impl quote::ToTokens,
            ) -> proc_macro2::TokenStream {
                let ident = &field.ident;
                let value = if field.nested {
                    value::nested(field, input_state_ty)
                } else {
                    value::simple(name, field)
                };
                quote! { #ident: #value }
            }

            mod value {
                use super::*;

                /// Generates value expression for simple fields.
                /// Example: `self.name.ok_or(MissingField { .. })?`
                pub fn simple(
                    name: &str,
                    FieldData { ident, ty, default, .. }: &FieldData,
                ) -> proc_macro2::TokenStream {
                    if matches_vec_signature(ty) || matches_option_signature(ty) {
                        quote! { self.#ident }
                    } else if *default {
                        quote! { self.#ident.unwrap_or_default() }
                    } else {
                        quote! {
                            self.#ident.ok_or(
                                axum_typed_multipart::TypedMultipartError::MissingField {
                                    field_name: String::from(#name)
                                }
                            )?
                        }
                    }
                }

                /// Generates value expression that finalizes nested builder.
                /// Example: `MultipartBuilder::finalize(self.addr)?`
                pub fn nested(
                    FieldData { ident, default, .. }: &FieldData,
                    input_state_ty: &impl quote::ToTokens,
                ) -> proc_macro2::TokenStream {
                    let finalize = quote! { axum_typed_multipart::MultipartBuilder::<#input_state_ty>::finalize(self.#ident) };
                    if *default {
                        quote! { #finalize.unwrap_or_default() }
                    } else {
                        quote! { #finalize? }
                    }
                }
            }
        }
    }
}
