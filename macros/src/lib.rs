mod util;

use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use util::get_option_type;

#[derive(Debug, FromField)]
#[darling(attributes(form_data))]
struct FieldData {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    field_name: Option<String>,
    #[darling(default)]
    default: bool,
}

impl FieldData {
    /// Get the name of the field from the `field_name` attribute, falling back
    /// to the field identifier.
    fn name(&self) -> String {
        self.field_name.to_owned().unwrap_or_else(|| self.ident.as_ref().unwrap().to_string())
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(form_data), supports(struct_named))]
struct InputData {
    ident: syn::Ident,
    data: darling::ast::Data<(), FieldData>,
}

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
#[proc_macro_error]
#[proc_macro_derive(TryFromMultipart, attributes(form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data } = match InputData::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => abort!(input, err.to_string()),
    };

    let fields = data.take_struct().unwrap();

    let declarations = fields.iter().map(|FieldData { ident, ty, default, .. }| {
        // When the value is an option we want to extract the inner type.
        let ty = get_option_type(ty).unwrap_or(ty);

        let value = if *default {
            quote! { Some(#ty::default()) }
        } else {
            quote! { None }
        };

        quote! {
            let mut #ident: core::option::Option<#ty> = #value;
        }
    });

    let assignments = fields.iter().map(|field @ FieldData { ident, .. }| {
        let name = field.name();
        quote! {
            if __field__name__ == #name {
                #ident = Some(
                    axum_typed_multipart::TryFromField::try_from_field(__field__).await?
                );
            }
        }
    });

    // We want to throw an error when a field is missing only if the former is
    // not an Option.
    let required_fields =
        fields.iter().filter(|FieldData { ty, .. }| get_option_type(ty).is_none());

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
                    let __field__name__ = __field__.name().unwrap().to_string();
                    #(#assignments) else *
                }

                #(#checks)*

                Ok(Self { #(#idents),* })
            }
        }
    };

    output.into()
}
