//! Macros for axum-typed-multipart.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod case_conversion;
mod derive_input;
mod impls;
mod limit_bytes;
mod util;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

#[proc_macro_error]
#[proc_macro_derive(TryFromMultipart, attributes(try_from_multipart, form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    impls::try_from_multipart::macro_impl(input)
}

#[proc_macro_error]
#[proc_macro_derive(TryFromField, attributes(try_from_field, field))]
pub fn try_from_field_derive(input: TokenStream) -> TokenStream {
    impls::try_from_field::macro_impl(input)
}
