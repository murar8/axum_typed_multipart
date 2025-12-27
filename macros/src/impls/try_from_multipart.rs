use crate::derive_input::InputData;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;

/// Derive the `TryFromMultipart` trait for arbitrary named structs.
pub fn macro_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let input = match InputData::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => return err.write_errors().into(),
    };

    let builder = crate::impls::multipart_builder::gen::output(input.clone());

    let ident = &input.ident;
    let generic = input.generic();
    let state_ty = input.state_ty();

    let builder_ident = input.builder_ident();
    let builder_ident =
        quote! { <#builder_ident as axum_typed_multipart::MultipartBuilder<#state_ty>> };

    quote! {
        #builder

        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::TryFromMultipartWithState<#state_ty> for #ident {
            async fn try_from_multipart_with_state(
                multipart: &mut axum::extract::multipart::Multipart,
                state: &#state_ty,
            ) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                let mut __builder__ = Default::default();

                while let Some(__field__) = multipart.next_field().await? {
                    // Prefix with '.' so nested builders can require delimiter
                    let __name__ = __field__.name().map(|n| format!(".{}", n));
                    let _ = #builder_ident::consume(&mut __builder__, __field__, __name__.as_deref(), state).await?;
                }

                #builder_ident::finalize(__builder__)
            }
        }
    }.into()
}
