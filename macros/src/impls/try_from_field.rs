use darling::{FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{Lit, LitStr};

use crate::{case_conversion::RenameCase, util::strip_leading_rawlit};

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
            let ident_stringified = strip_leading_rawlit(&self.ident.to_string());

            // .map_or_else will require cloning `ident_stringified` in any way
            if let Some(case) = rename_all {
                case.convert_case(&ident_stringified)
            } else {
                ident_stringified
            }
        }
    }
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
pub fn macro_impl(input: TokenStream) -> TokenStream {
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
