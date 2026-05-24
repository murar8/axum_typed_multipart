//! Verifies that the derive macros emit fully-qualified paths and do not
//! depend on the prelude or on any specific names being in scope.
//!
//! `#![no_implicit_prelude]` disables the Rust prelude, so any unqualified
//! reference in the macro output (e.g. `Some`, `Result`, `String`, `Vec`)
//! will fail to compile.

#![no_implicit_prelude]
#![allow(dead_code)]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

// `#[async_trait]` (third-party) requires these in scope; our own macros
// must not require anything beyond what they qualify themselves.
extern crate alloc as __alloc;
use ::core::option::Option::None;
use __alloc::boxed::Box;

#[derive(::axum_typed_multipart::TryFromMultipart)]
struct Lax {
    name: ::std::string::String,
    tags: ::std::vec::Vec<::std::string::String>,
    opt: ::core::option::Option<::std::string::String>,
    #[form_data(default)]
    def: ::std::string::String,
}

#[derive(::axum_typed_multipart::TryFromMultipart)]
#[try_from_multipart(strict)]
struct Strict {
    name: ::std::string::String,
    tags: ::std::vec::Vec<::std::string::String>,
    opt: ::core::option::Option<::std::string::String>,
}

#[derive(::axum_typed_multipart::TryFromField)]
enum Plain {
    A,
    B,
}

#[derive(::axum_typed_multipart::TryFromField)]
#[try_from_field(rename_all = "snake_case")]
enum Renamed {
    #[field(rename = "x")]
    First,
    Second,
}
