use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(try_from_multipart), supports(struct_named))]
struct InputData {
    ident: syn::Ident,

    #[darling(default)]
    strict: bool,

    /// Accepted but handled by the builder macro.
    #[darling(default)]
    #[allow(dead_code)]
    rename_all: Option<String>,

    #[darling(default)]
    state: Option<syn::Path>,
}

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let InputData { ident, strict, state, .. } = match InputData::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => abort!(input, err.to_string()),
    };

    let builder_ident = syn::Ident::new(&format!("{}Builder", ident), ident.span());

    let missing_field_name_handling = if strict {
        quote! { return Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
    } else {
        quote! { continue }
    };

    let unknown_field_handling = if strict {
        quote! {
            if !__processed__ {
                return Err(axum_typed_multipart::TypedMultipartError::UnknownField {
                    field_name: __field_name__
                });
            }
        }
    } else {
        quote! {}
    };

    let generic = state.is_none().then(|| quote! { <S: Sync> });
    let state = state.map(|state| quote! { #state }).unwrap_or(quote! { S });

    let output = quote! {
        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::TryFromMultipartWithState<#state> for #ident {
            async fn try_from_multipart_with_state(multipart: &mut axum::extract::multipart::Multipart, state: &#state) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                let mut __builder__ = #builder_ident::default();

                while let Some(__field__) = multipart.next_field().await? {
                    let __field_name__ = match __field__.name() {
                        | Some("")
                        | None => #missing_field_name_handling,
                        | Some(name) => name.to_string(),
                    };

                    let __processed__ = __builder__.process_field(__field__, state).await?;
                    #unknown_field_handling
                }

                __builder__.build()
            }
        }
    };

    output.into()
}
