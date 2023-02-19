use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Ident, Type};

struct FieldData {
    ident: Ident,
    ty: Type,
    name: String,
}

#[proc_macro_derive(TryFromMultipart)]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let fields = match data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => panic!("input must be a struct"),
    };

    let get_field_data = |Field { ident, ty, .. }| match ident {
        Some(ident) => FieldData {
            name: ident.clone().to_string(),
            ident,
            ty,
        },
        None => {
            panic!("tuple structs are not supported")
        }
    };

    let field_data = fields.into_iter().map(get_field_data).collect::<Vec<_>>();

    let declarations = field_data.iter().map(|FieldData { ident, ty, .. }| {
        quote! {
            let mut #ident: Option<#ty> = None;
        }
    });

    let assignments = field_data.iter().map(|FieldData { ident, ty, name }| {
        quote! {
            if __field__.name().unwrap().to_string() == #name {
                #ident = Some(
                    <#ty as axum_typed_multipart::TryFromField>::try_from_field(__field__).await?
                );
            }
        }
    });

    let checks = field_data.iter().map(|FieldData { ident, name, .. }| {
        quote! {
            let #ident = #ident.ok_or(
                axum_typed_multipart::TypedMultipartError::MissingField(
                    String::from(#name)
                )
            )?;
        }
    });

    let idents = field_data.iter().map(|FieldData { ident, .. }| ident);

    let output = quote! {
        #[axum::async_trait]
        impl axum_typed_multipart::TryFromMultipart for #ident {
            async fn try_from_multipart(mut multipart: axum::extract::Multipart) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                #(#declarations)*

                while let Some(__field__) = multipart.next_field().await? {
                    #(#assignments) else *
                }

                #(#checks)*

                Ok(Self { #(#idents),* })
            }
        }
    };

    output.into()
}
