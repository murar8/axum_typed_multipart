use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;

#[derive(Debug, FromField)]
#[darling(attributes(form_data))]
struct FieldData {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    field_name: Option<String>,
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

/// Derive the [TryFromMultipart] trait for arbitrary named structs.
///
/// All fields for the supplied struct must implement the [TryFromField] trait
/// to be able to derive the trait.
///
/// ## Attributes
///
/// ### `form_data`
///
/// Can be applied to the struct fields to configure the parser behaviour.
///
/// #### Arguments
///
/// - `field_name` => Can be used to configure a different name for the source
///    field in the incoming request.
#[proc_macro_error]
#[proc_macro_derive(TryFromMultipart, attributes(form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, data } = match InputData::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => abort!(input, err.to_string()),
    };

    let fields = data.take_struct().unwrap();

    let declarations = fields.iter().map(|FieldData { ident, ty, .. }| {
        quote! {
            let mut #ident: Option<#ty> = None;
        }
    });

    let assignments = fields.iter().map(|field @ FieldData { ident, ty, .. }| {
        let name = field.name();
        quote! {
            if __field__name__ == #name {
                #ident = Some(
                    <#ty as axum_typed_multipart::TryFromField>::try_from_field(__field__).await?
                );
            }
        }
    });

    let checks = fields.iter().map(|field @ FieldData { ident, .. }| {
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
            async fn try_from_multipart(mut multipart: axum::extract::Multipart) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
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
