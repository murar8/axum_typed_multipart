mod util;

use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use ubyte::ByteUnit;
use util::{matches_option_signature, matches_vec_signature};

const DEFAULT_FIELD_SIZE_LIMIT_BYTES: usize = 1024 * 1024; // 1MiB

#[derive(Debug, FromField)]
#[darling(attributes(form_data))]
struct FieldData {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    field_name: Option<String>,

    #[darling(default)]
    limit: Option<String>,

    #[darling(default)]
    default: bool,
}

impl FieldData {
    /// Get the name of the field from the `field_name` attribute, falling back
    /// to the field identifier.
    fn name(&self) -> String {
        if let Some(field_name) = &self.field_name {
            return field_name.to_string();
        }

        let ident = self.ident.as_ref().unwrap().to_string();

        if ident.starts_with("r#") {
            // If the field is using a raw identifier we want to strip the
            // leading characters.
            ident.chars().skip(2).collect()
        } else {
            ident
        }
    }

    /// Parse the supplied human readable size limit into a byte limit.
    fn limit_bytes(&self) -> Option<usize> {
        match self.limit.as_deref() {
            None => Some(DEFAULT_FIELD_SIZE_LIMIT_BYTES),
            Some("unlimited") => None,
            Some(limit) => match limit.parse::<ByteUnit>() {
                Ok(limit) => Some(limit.as_u64() as usize),
                Err(_) => abort!(self.ident.as_ref().unwrap(), "limit must be a valid byte unit"),
            },
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(try_from_multipart), supports(struct_named))]
struct InputData {
    ident: syn::Ident,

    data: darling::ast::Data<(), FieldData>,

    #[darling(default)]
    strict: bool,
}

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
#[proc_macro_error]
#[proc_macro_derive(TryFromMultipart, attributes(try_from_multipart, form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data, strict } = match InputData::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => abort!(input, err.to_string()),
    };

    let fields = data.take_struct().unwrap();

    let declarations = fields.iter().map(|FieldData { ident, ty, default, .. }| {
         if matches_vec_signature(ty) {
            quote! { let mut #ident: #ty = std::vec::Vec::new(); }
        } else if matches_option_signature(ty) {
            quote! { let mut #ident: #ty = std::option::Option::None; }
        } else if *default {
            quote! { let mut #ident: std::option::Option<#ty> = std::option::Option::Some(#ty::default()); }
        } else {
            quote! { let mut #ident: std::option::Option<#ty> = std::option::Option::None; }
        }
    });

    let mut assignments = fields
        .iter()
        .map(|field @ FieldData { ident, ty, .. }| {
            let name = field.name();
            let limit_bytes =
                field.limit_bytes().map(|limit| quote! { Some(#limit) }).unwrap_or(quote! { None });
            let value = quote! {
                axum_typed_multipart::TryFromField::try_from_field(__field__, #limit_bytes).await?
            };

            let assignment = if matches_vec_signature(ty) {
                quote! { #ident.push(#value); }
            } else if strict {
                quote! {
                    if #ident.is_none() {
                        #ident = Some(#value);
                    } else {
                        return Err(
                            axum_typed_multipart::TypedMultipartError::DuplicateField {
                                field_name: String::from(#name)
                            }
                        );
                    }
                }
            } else {
                quote! { #ident = Some(#value); }
            };

            quote! {
                if __field_name__ == #name {
                    #assignment
                }
            }
        })
        .collect::<Vec<_>>();

    if strict {
        assignments.push(quote! {
            {
                return Err(
                    axum_typed_multipart::TypedMultipartError::UnknownField {
                        field_name: __field_name__
                    }
                );
            }
        })
    }

    let required_fields = fields
        .iter()
        .filter(|FieldData { ty, .. }| !matches_option_signature(ty) && !matches_vec_signature(ty));

    let checks = required_fields.map(|field @ FieldData { ident, .. }| {
        let field_name = field.name();
        quote! {
            let #ident = #ident.ok_or(
                axum_typed_multipart::TypedMultipartError::MissingField {
                    field_name: String::from(#field_name)
                }
            )?;
        }
    });

    let idents = fields.iter().map(|FieldData { ident, .. }| ident);

    let output = quote! {
        #[axum::async_trait]
        impl axum_typed_multipart::TryFromMultipart for #ident {
            async fn try_from_multipart(multipart: &mut axum::extract::Multipart) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                #(#declarations)*

                while let Some(__field__) = multipart.next_field().await? {
                    let __field_name__ = __field__.name().unwrap().to_string();
                    #(#assignments) else *
                }

                #(#checks)*

                Ok(Self { #(#idents),* })
            }
        }
    };

    output.into()
}
