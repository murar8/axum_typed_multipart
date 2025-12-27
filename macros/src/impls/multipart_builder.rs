use crate::case_conversion::RenameCase;
use crate::derive_input::{FieldData, InputData};
use crate::limit_bytes::LimitBytes;
use crate::util::{
    extract_inner_type, matches_option_signature, matches_vec_signature, strip_leading_rawlit,
};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;

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

        // Compute field names once, map name → field
        let fields: HashMap<_, _> = fields.iter().map(|f| (field_name(f, rename_all), f)).collect();

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
        fields: &HashMap<String, &FieldData>,
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
        pub fn field(field @ FieldData { nested, .. }: &FieldData) -> proc_macro2::TokenStream {
            if !nested {
                field::simple(field)
            } else {
                field::nested(field)
            }
        }

        mod field {
            use super::*;

            /// Generates builder field for simple types.
            /// Example: `name: String` → `name: Option<String>`
            /// Example: `names: Vec<String>` → `names: Vec<String>`
            pub fn simple(FieldData { ident, ty, .. }: &FieldData) -> proc_macro2::TokenStream {
                if matches_vec_signature(ty) || matches_option_signature(ty) {
                    quote! { #ident: #ty }
                } else {
                    quote! { #ident: std::option::Option<#ty> }
                }
            }

            /// Generates builder field for nested structs.
            /// Example: `addr: Address` → `addr: AddressMultipartBuilder`
            /// Example: `addrs: Vec<Address>` → `addrs: Vec<AddressMultipartBuilder>`
            /// Example: `addr: Option<Address>` → `addr: Option<AddressMultipartBuilder>`
            pub fn nested(FieldData { ident, ty, .. }: &FieldData) -> proc_macro2::TokenStream {
                let inner_ty = if matches_vec_signature(ty) || matches_option_signature(ty) {
                    extract_inner_type(ty)
                } else {
                    ty
                };
                let field_builder_ident = builder_ident(inner_ty);
                if matches_vec_signature(ty) {
                    quote! { #ident: Vec<#field_builder_ident> }
                } else if matches_option_signature(ty) {
                    quote! { #ident: Option<#field_builder_ident> }
                } else {
                    quote! { #ident: #field_builder_ident }
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
        fields: &HashMap<String, &FieldData>,
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
            fields: &HashMap<String, &FieldData>,
            strict: bool,
        ) -> proc_macro2::TokenStream {
            let branches = fields.iter().map(|(name, field)| consume::branch(name, field, strict));

            let on_nameless_field = if !strict {
                quote! { Ok(Some(__field__)) }
            } else {
                quote! { Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
            };

            let on_unmatched_field = if !strict {
                quote! { Ok(Some(__field__)) }
            } else {
                quote! {
                    Err(axum_typed_multipart::TypedMultipartError::UnknownField {
                        field_name: __name__.to_string()
                    })
                }
            };

            quote! {
                async fn consume<'a>(
                    &mut self,
                    __field__: axum::extract::multipart::Field<'a>,
                    __name__: Option<&str>,
                    __state__: &#input_state_ty,
                ) -> Result<Option<axum::extract::multipart::Field<'a>>, axum_typed_multipart::TypedMultipartError> {
                    let __name__ = match __name__ {
                        None | Some("") => return #on_nameless_field,
                        Some(__name__) => __name__,
                    };
                    #(#branches)*
                    #on_unmatched_field
                }
            }
        }

        /// Helpers for generating consume method branches.
        mod consume {
            use super::*;

            /// Dispatches to simple or nested branch generation.
            pub fn branch(name: &str, field: &FieldData, strict: bool) -> proc_macro2::TokenStream {
                if !field.nested {
                    branch::simple(name, field, strict)
                } else {
                    branch::nested(name, field)
                }
            }

            mod branch {
                use super::*;

                /// Generates match branch for simple field consumption.
                /// Example: `if __name__ == "name" { self.name = Some(value); return Ok(None); }`
                pub fn simple(
                    name: &str,
                    FieldData { ident, ty, limit, .. }: &FieldData,
                    strict: bool,
                ) -> proc_macro2::TokenStream {
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
                                        field_name: String::from(#name)
                                    });
                                }
                            }
                        }
                    };

                    quote! {
                        if __name__ == #name {
                            #assignment
                            return Ok(None);
                        }
                    }
                }

                /// Generates match branch that delegates to nested builder.
                /// Example: `if let Some(rest) = name.strip_prefix("addr.") { self.addr.consume(.., rest); }`
                pub fn nested(
                    name: &str,
                    FieldData { ident, ty, .. }: &FieldData,
                ) -> proc_macro2::TokenStream {
                    let prefix = if matches_vec_signature(ty) {
                        name.to_owned()
                    } else {
                        format!("{name}.")
                    };
                    quote! {
                        if let Some(__rest__) = __name__.strip_prefix(#prefix) {
                            return self.#ident.consume(__field__, Some(__rest__), __state__).await;
                        }
                    }
                }
            }
        }

        /// Generates: `fn finalize(self) -> Result<Foo, _> { Ok(Foo { .. }) }`
        pub fn finalize(
            ident: &syn::Ident,
            fields: &HashMap<String, &FieldData>,
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
                if !field.nested {
                    field::simple(name, field)
                } else {
                    field::nested(field, input_state_ty)
                }
            }

            mod field {
                use super::*;

                /// Generates field assignment for finalize.
                /// Example: `name: self.name.ok_or(MissingField { .. })?`
                /// Example: `name: self.name.unwrap_or_default()` (if #[default])
                pub fn simple(
                    name: &str,
                    FieldData { ident, ty, default, .. }: &FieldData,
                ) -> proc_macro2::TokenStream {
                    if matches_vec_signature(ty) || matches_option_signature(ty) {
                        quote! { #ident: self.#ident }
                    } else if *default {
                        quote! { #ident: self.#ident.unwrap_or_default() }
                    } else {
                        quote! {
                            #ident: self.#ident.ok_or(
                                axum_typed_multipart::TypedMultipartError::MissingField {
                                    field_name: String::from(#name)
                                }
                            )?
                        }
                    }
                }

                /// Generates field assignment that finalizes nested builder.
                /// Example: `addr: MultipartBuilder::finalize(self.addr)?`
                pub fn nested(
                    FieldData { ident, default, .. }: &FieldData,
                    input_state_ty: &impl quote::ToTokens,
                ) -> proc_macro2::TokenStream {
                    let finalize = quote! { axum_typed_multipart::MultipartBuilder::<#input_state_ty>::finalize(self.#ident) };
                    if *default {
                        quote! { #ident: #finalize.unwrap_or_default() }
                    } else {
                        quote! { #ident: #finalize? }
                    }
                }
            }
        }
    }
}
