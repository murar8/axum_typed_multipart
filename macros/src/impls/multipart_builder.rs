use crate::derive_input::{FieldData, InputData};
use crate::limit_bytes::LimitBytes;
use crate::util::{matches_option_signature, matches_vec_signature};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;

pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match InputData::from_derive_input(&input) {
        Ok(input) => generate(input).into(),
        Err(err) => return err.write_errors().into(),
    }
}

pub fn generate(input: InputData) -> impl Into<TokenStream> + darling::ToTokens {
    let generic = input.generic();
    let state_ty = input.state_ty();
    let builder_ident = input.builder_ident();

    let InputData { ident, data, strict, rename_all, .. } = input;
    let fields = data.take_struct().unwrap();

    let builder_fields = fields.iter().map(|FieldData { ident, ty, .. }| {
        if matches_vec_signature(ty) || matches_option_signature(ty) {
            quote! { #ident: #ty }
        } else {
            quote! { #ident: std::option::Option<#ty> }
        }
    });

    let consume_branches = fields.iter().map(|field| {
        let ident = &field.ident;
        let name = field.name(rename_all);
        let limit_bytes = field.limit.unwrap_or(LimitBytes(None));

        let value = quote! {
            <_ as axum_typed_multipart::TryFromFieldWithState<_>>::try_from_field_with_state(__field__, #limit_bytes, __state__).await?
        };

        let assignment = if matches_vec_signature(&field.ty) {
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
            if __field_name__ == #name {
                #assignment
                return Ok(None);
            }
        }
    });

    let finalize_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let name = field.name(rename_all);

        if matches_vec_signature(&field.ty) || matches_option_signature(&field.ty) {
            quote! { #ident: self.#ident }
        } else if field.default {
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
    });

    let on_unmatched_field = if !strict {
        quote! { Ok(Some(__field__)) }
    } else {
        quote! {
            Err(axum_typed_multipart::TypedMultipartError::UnknownField {
                field_name: __field_name__
            })
        }
    };

    quote! {
        #[derive(Default)]
        struct #builder_ident {
            #(#builder_fields),*
        }

        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::MultipartBuilder<#state_ty> for #builder_ident {
            type Target = #ident;

            async fn consume<'a>(
                &mut self,
                __field__: axum::extract::multipart::Field<'a>,
                __state__: &#state_ty,
            ) -> Result<Option<axum::extract::multipart::Field<'a>>, axum_typed_multipart::TypedMultipartError> {
                let __field_name__ = __field__.name().unwrap_or("").to_string();
                #(#consume_branches)*
                #on_unmatched_field
            }

            fn finalize(self) -> Result<Self::Target, axum_typed_multipart::TypedMultipartError> {
                Ok(#ident { #(#finalize_fields),* })
            }
        }
    }
}
