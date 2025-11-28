//! Macros for axum-typed-multipart.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod case_conversion;
mod impls;
mod util;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

#[proc_macro_error]
#[proc_macro_derive(TryFromMultipart, attributes(try_from_multipart, form_data))]
pub fn try_from_multipart_derive(input: TokenStream) -> TokenStream {
    let builder = impls::try_from_multipart_builder::macro_impl(input.clone());
    let trait_impl = impls::try_from_multipart::macro_impl(input);
    [builder, trait_impl].into_iter().collect()
}

#[proc_macro_error]
#[proc_macro_derive(TryFromField, attributes(try_from_field, field))]
pub fn try_from_field_derive(input: TokenStream) -> TokenStream {
    impls::try_from_field::macro_impl(input)
}

#[proc_macro_error]
#[proc_macro_derive(TryFromMultipartBuilder, attributes(try_from_multipart, form_data))]
pub fn try_from_multipart_builder_derive(input: TokenStream) -> TokenStream {
    impls::try_from_multipart_builder::macro_impl(input)
}
