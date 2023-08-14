//! Macros for axum-typed-multipart.

mod case_conversion;
mod util;

use case_conversion::RenameCase;

use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{Lit, LitStr};
use ubyte::ByteUnit;
use util::{matches_option_signature, matches_vec_signature};

const DEFAULT_FIELD_SIZE_LIMIT_BYTES: usize = 1024 * 1024; // 1MiB

#[derive(Debug, FromVariant)]
#[darling(attributes(field))]
struct FieldEnumData {
    ident: syn::Ident,

    #[darling(default)]
    rename: Option<String>,
}

impl FieldEnumData {
    pub fn name(&self, rename_all: Option<RenameCase>) -> String {
        if let Some(rename) = self.rename.clone() {
            rename
        } else {
            let ident_stringified = util::strip_leading_rawlit(&self.ident.to_string());

            // .map_or_else will require cloning `ident_stringified` in any way
            if let Some(case) = rename_all {
                case.convert_case(&ident_stringified)
            } else {
                ident_stringified
            }
        }
    }
}

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
    fn name(&self, rename_all: Option<RenameCase>) -> String {
        if let Some(field_name) = &self.field_name {
            return field_name.to_string();
        }

        let ident = self.ident.as_ref().unwrap().to_string();
        let field_in_struct = util::strip_leading_rawlit(&ident);

        if let Some(case_conversion) = rename_all {
            case_conversion.convert_case(&field_in_struct)
        } else {
            field_in_struct
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

    #[darling(default)]
    rename_all: Option<String>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(try_from_field), supports(enum_unit))]
struct TryFromFieldInputData {
    ident: syn::Ident,
    data: darling::ast::Data<FieldEnumData, ()>,

    #[darling(default)]
    rename_all: Option<String>,
}

/// Derive `TryFromField` for arbitrary unit-enums.
#[proc_macro_error]
#[proc_macro_derive(TryFromField, attributes(try_from_field, field))]
pub fn try_from_field(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let TryFromFieldInputData { ident, data, rename_all } =
        match TryFromFieldInputData::from_derive_input(&input) {
            Ok(input) => input,
            Err(err) => abort!(input, err.to_string()),
        };
    let fields = data.take_enum().unwrap();
    let rename_all = RenameCase::from_option_fallible(&ident, rename_all);

    let match_arms = fields.iter().map(|f| {
        let name = f.name(rename_all);
        let f_ident = &f.ident;
        let strlit = Lit::Str(LitStr::new(&name, f_ident.span()));
        quote! {
            #strlit => Ok(Self::#f_ident)
        }
    });

    quote! {
        #[axum::async_trait]
        impl ::axum_typed_multipart::TryFromField for #ident {
            async fn try_from_field(
                field: ::axum::extract::multipart::Field<'_>,
                limit_bytes: ::core::option::Option<usize>,
            ) -> ::core::result::Result<Self, ::axum_typed_multipart::TypedMultipartError> {
                let string: String = ::axum_typed_multipart::TryFromField::try_from_field(field, limit_bytes).await?;
                match string.as_str() {
                    #(#match_arms),*,
                    _ => Err(::axum_typed_multipart::TypedMultipartError::UnknownField {
                        field_name: string
                    })
                }
            }
        }
    }
    .into()
}

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
#[proc_macro_error]
#[proc_macro_derive(TryFromMultipart, attributes(try_from_multipart, form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data, strict, rename_all } = match InputData::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => abort!(input, err.to_string()),
    };
    let rename_all = RenameCase::from_option_fallible(&ident, rename_all);

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
            let name = field.name(rename_all);
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
        let field_name = field.name(rename_all);
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
