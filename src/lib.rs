//! Helper library for the [axum framework](https://github.com/tokio-rs/axum)
//! designed to allow you to parse the `multipart/form-data` body of the
//! supplied request into an arbitrary struct.
//!
//! **This library is still under heavy development and is subject to change
//! without notice.**
//!
//! ## Usage
//!
//! ### Installation
//!
//! ```bash
//! cargo add axum_typed_multipart
//! ```
//!
//! ### Getting started
//!
//! To get started you will need to define a struct with the desired fields and
//! implement the [TryFromMultipart](crate::TryFromMultipart) trait. In the vast
//! majority of cases you will want to use the derive macro to generate the
//! implementation automatically.
//! To be able to derive the implementation every field must implement the
//! [TryFromField](crate::TryFromField) trait. The trait is
//! implemented by default for all primitive types, [`String`], and [`Vec<u8>`]
//! in case you just want to access the raw bytes.
//!
//! If the request body is malformed or it does not contain the necessary data
//! the request will be aborted with an error.
//!
//! ```rust
//! use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     first_name: String,
//!     last_name: String,
//! }
//!
//! async fn handler(
//!     TypedMultipart(RequestData { first_name, last_name }): TypedMultipart<RequestData>,
//! ) {
//!     // All fields are guaranteed to be populated.
//! }
//! ```
//!
//! ### Optional fields
//!
//! If a field is declared as an [Option] the value will default to
//! [Option::None] when the field is missing from the request body.
//!
//! ```rust
//! use axum_typed_multipart::TryFromMultipart;
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     first_name: Option<String>,
//! }
//! ```
//!
//! ### Renaming fields
//!
//! If you would like to assign a custom name to the struct field you can use
//! the `field_name` parameter in the `form_data` attribute.
//!
//! ```rust
//! use axum_typed_multipart::TryFromMultipart;
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     #[form_data(field_name = "input_file")]
//!     name: Option<String>,
//! }
//! ```
//!
//! ### Field metadata
//!
//! If you need access to the field metadata (e.g. the request headers) you can
//! use the [FieldData](crate::FieldData) struct to wrap your field.
//!
//! ```rust
//! use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     image: FieldData<Vec<u8>>,
//! }
//!
//! async fn handler(TypedMultipart(RequestData { image }): TypedMultipart<RequestData>) {
//!     println!("content_type = {}", image.metadata.content_type.unwrap());
//! }
//! ```
//!
//! ### Large uploads
//!
//! For large file uploads you can save the contents of the file to the file
//! system using the [TempFile](crate::TempFile) helper. This will stream the
//! field body to the file system allowing you to save the contents later.
//!
//! ```rust
//! use axum_typed_multipart::{FieldData, TempFile, TryFromMultipart, TypedMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     image: FieldData<TempFile>,
//! }
//!
//! async fn handler(TypedMultipart(RequestData { image }): TypedMultipart<RequestData>) {
//!     image.contents.persist("/data/file.bin", false).await.unwrap();
//! }
//! ```

mod field_data;
mod field_metadata;
mod temp_file;
mod try_from_field;
mod try_from_multipart;
mod typed_multipart;
mod typed_multipart_error;

pub use crate::field_data::FieldData;
pub use crate::field_metadata::FieldMetadata;
pub use crate::temp_file::TempFile;
pub use crate::try_from_field::TryFromField;
pub use crate::try_from_multipart::TryFromMultipart;
pub use crate::typed_multipart::TypedMultipart;
pub use crate::typed_multipart_error::TypedMultipartError;
pub use axum_typed_multipart_macros::TryFromMultipart;
