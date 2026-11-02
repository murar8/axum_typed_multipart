//! Macros for axum-typed-multipart.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod case_conversion;
mod impls;
mod limit_bytes;
mod util;

use proc_macro::TokenStream;

/// Derives multipart form parsing for a struct using its field types and form attributes.
#[proc_macro_derive(TryFromMultipart, attributes(try_from_multipart, form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    impls::try_from_multipart::macro_impl(input)
}

/// Derives multipart field parsing for an enum using its variant names and field attributes.
#[proc_macro_derive(TryFromField, attributes(try_from_field, field))]
pub fn try_from_field_derive(input: TokenStream) -> TokenStream {
    impls::try_from_field::macro_impl(input)
}
