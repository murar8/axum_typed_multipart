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
    let strict = input.strict;
    let generic = input.generic();
    let state_ty = input.state_ty();

    let builder_ident = input.builder_ident();
    // cannot infer state type here, so we have to be explicit
    let builder_ident =
        quote! { <#builder_ident as axum_typed_multipart::MultipartBuilder<#state_ty>> };

    let on_nameless_field = if !strict {
        quote! { continue }
    } else {
        quote! { return Err(axum_typed_multipart::TypedMultipartError::NamelessField) }
    };

    let on_unmatched_field = if !strict {
        quote! { continue }
    } else {
        quote! {
            return Err(axum_typed_multipart::TypedMultipartError::UnknownField {
                field_name: __field__.name().unwrap_or_default().to_string()
            })
        }
    };

    let consume_loop = quote! {
        while let Some(__field__) = multipart.next_field().await? {
            let __name__ = match __field__.name() {
                None | Some("") => { #on_nameless_field }
                Some(name) => format!(".{}", name), // Prefix with '.' so nested builders can require delimiter
            };
            if let Some(__field__) = #builder_ident::consume(&mut __builder__, __field__, &__name__, state).await? {
                #on_unmatched_field
            }
        }
    };

    let impl_block = quote! {
        #[axum_typed_multipart::async_trait]
        impl #generic axum_typed_multipart::TryFromMultipartWithState<#state_ty> for #ident {
            async fn try_from_multipart_with_state(
                multipart: &mut axum::extract::multipart::Multipart,
                state: &#state_ty,
            ) -> Result<Self, axum_typed_multipart::TypedMultipartError> {
                let mut __builder__ = Default::default();
                #consume_loop
                #builder_ident::finalize(__builder__)
            }
        }
    };

    quote! {
        #builder
        #impl_block
    }
    .into()
}
