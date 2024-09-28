//! Designed to seamlessly integrate with [Axum](https://github.com/tokio-rs/axum), this crate
//! simplifies the process of handling `multipart/form-data` requests in your web application by
//! allowing you to parse the request body into a type-safe struct.
//!
//! ## Installation
//!
//! ```bash
//! cargo add axum_typed_multipart
//! ```
//!
//! ### Features
//!
//! All features are enabled by default.
//!
//! - `chrono_0_4`: Enables support for [chrono::DateTime](chrono_0_4::DateTime) (v0.4)
//! - `tempfile_3`: Enables support for [tempfile::NamedTempFile](tempfile_3::NamedTempFile) (v3)
//! - `uuid_1`: Enables support for [uuid::Uuid](uuid_1::Uuid) (v1)
//!
//! ## Usage
//!
//! ### Getting started
//!
//! To get started you will need to define a struct with the desired fields and implement the
//! [TryFromMultipart](crate::TryFromMultipart) trait. In the vast majority of cases you will want
//! to use the derive macro to generate the implementation automatically.
//!
//! To be able to derive the [TryFromMultipart](crate::TryFromMultipart) trait every field in the
//! struct must implement the [TryFromField](crate::TryFromField) trait.
//!
//! The [TryFromField](crate::TryFromField) trait is implemented by default for the following
//! types:
//! - [i8], [i16], [i32], [i64], [i128], [isize]
//! - [u8], [u16], [u32], [u64], [u128], [usize]
//! - [f32], [f64]
//! - [bool]
//! - [char]
//! - [String]
//! - [axum::body::Bytes]
//! - [chrono::DateTime](chrono_0_4::DateTime) (feature: `chrono_0_4`)
//! - [tempfile::NamedTempFile](tempfile_3::NamedTempFile) (feature: `tempfile_3`)
//! - [uuid::Uuid](uuid_1::Uuid) (feature: `uuid_1`)
//!
//! If the request body is malformed the request will be aborted with an error.
//!
//! An error will be returned if at least one field is missing, with the exception of [Option] and
//! [Vec] types, which will be set respectively as [Option::None] and `[]`.
//!
//! ```rust,no_run
#![doc = include_str!("../examples/basic.rs")]
//! ```
//!
//! ### Optional fields
//!
//! If a field is declared as an [Option] the value will default to [None] when the field is
//! missing from the request body.
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
//! If you would like to assign a custom name for the source field you can use the `field_name`
//! parameter of the `form_data` attribute.
//! ```rust
//! use axum_typed_multipart::TryFromMultipart;
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     #[form_data(field_name = "first_name")]
//!     name: Option<String>,
//! }
//! ```
//!
//! The `rename_all` parameter from the `try_from_multipart` attribute can be used to automatically
//! rename each field of your struct to a specific case. It works the same way as
//! `#[serde(rename_all = "...")]`.
//!
//! Supported cases:
//! - `snake_case`
//! - `camelCase`
//! - `PascalCase`
//! - `kebab-case`
//! - `UPPERCASE`
//! - `lowercase`
//!
//!  ```rust
//! use axum_typed_multipart::TryFromMultipart;
//!
//! #[derive(TryFromMultipart)]
//! #[try_from_multipart(rename_all = "UPPERCASE")]
//! struct RequestData {
//!     name: Option<String>, // Will be renamed to `NAME` in the request.
//! }
//! ```
//!
//! NOTE: If the `#[form_data(field_name = "...")]` attribute is specified, the `rename_all` rule
//! will not be applied.
//!
//! ### Default values
//!
//! If the `default` parameter in the `form_data` attribute is present the value will be populated
//! using the type's [Default] implementation when the field is not supplied in the request.
//! ```rust
//! use axum_typed_multipart::TryFromMultipart;
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     #[form_data(default)]
//!     name: String, // defaults to ""
//! }
//! ```
//!
//! ### Field metadata
//!
//! If you need access to the field metadata (e.g. the field headers like file name or content
//! type) you can use the [FieldData](crate::FieldData) struct to wrap your field.
//! ```rust
//! use axum::body::Bytes;
//! use axum_typed_multipart::{FieldData, TryFromMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     image: FieldData<Bytes>,
//! }
//! ```
//!
//! ### Large uploads
//!
//! For large uploads you can save the contents of the field to the file system using
//! [tempfile::NamedTempFile](tempfile_3::NamedTempFile). This will efficiently stream the field
//! data directly to the file system, without needing to fit all the data in memory. Once the
//! upload is complete, you can then save the contents to a location of your choice. For more
//! information check out the [NamedTempFile](tempfile_3::NamedTempFile) documentation.
//!
//! #### **Warning**
//! Field size limits for [Vec] fields are applied to **each** occurrence of the field. This means
//! that if you have a 1GiB field limit and the field contains
//! 5 entries, the total size of the request body will be 5GiB.
//!
//! #### **Note**
//! When handling large uploads you will need to increase both the request body size limit and the
//! field size limit. The request body size limit can be increased using the
//! [DefaultBodyLimit](axum::extract::DefaultBodyLimit) middleware, while the field size limit can
//! be increased using the `limit` parameter of the `form_data` attribute.
//! ```rust,no_run
#![doc = include_str!("../examples/upload.rs")]
//! ```
//!
//! ### Lists
//!
//! If the incoming request will include multiple fields that share the same name (AKA lists) the
//! field can be declared as a [Vec], allowing for all occurrences of the field to be stored.
//!
//! #### **Warning**
//! Field size limits for [Vec] fields are applied to **each** occurrence of the field. This means
//! that if you have a 1GiB field limit and the field contains
//! 5 entries, the total size of the request body will be 5GiB.
//! ```rust
//! use axum::http::StatusCode;
//! use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     names: Vec<String>,
//! }
//! ```
//!
//! ### Strict mode
//!
//! By default the derive macro will store the last occurrence of a field and it will ignore
//! unknown fields. This behavior can be changed by using the `strict` parameter in the derive
//! macro. This will make the macro throw an error if the request contains multiple fields with the
//! same name or if it contains unknown fields. In addition when using strict mode sending fields
//! with a missing or empty name will result in an error.
//! ```rust
//! use axum_typed_multipart::TryFromMultipart;
//!
//! #[derive(TryFromMultipart)]
//! #[try_from_multipart(strict)]
//! struct RequestData {
//!     name: String,
//! }
//! ```
//!
//! ### Enums
//!
//! `axum_typed_multipart` also supports custom enum parsing by deriving the
//! [`TryFromField`] trait:
//! ```rust
//! use axum_typed_multipart::{TryFromField, TryFromMultipart};
//!
//! #[derive(TryFromField)]
//! enum Sex {
//!     Male,
//!     Female,
//! }
//!
//! #[derive(TryFromMultipart)]
//! struct Person {
//!     name: String,
//!     sex: Sex,
//! }
//! ```
//!
//! Enum fields can be renamed in two ways:
//! ```rust
//! use axum_typed_multipart::TryFromField;
//!
//! #[derive(TryFromField)]
//! // Using the `#[try_from_field(rename_all = "...")]` renaming attribute.
//! // It works the same as way as the `TryFromMultipart` implementation.
//! #[try_from_field(rename_all = "snake_case")]
//! enum AccountType {
//!     // Or using the `#[field(rename = "...")]` attribute.
//!     #[field(rename = "administrator")]
//!     Admin,
//!     Moderator,
//!     Plain
//! }
//! ```
//!
//! ### Custom types
//!
//! If you would like to use a custom type for a field you need to implement the
//! [TryFromField](crate::TryFromField) trait for your type. This will allow the derive macro to
//! generate the [TryFromMultipart](crate::TryFromMultipart) implementation automatically. Instead
//! of implementing the trait directly, it is recommended to implement the
//! [TryFromChunks](crate::TryFromChunks) trait and the [TryFromField](crate::TryFromField) trait
//! will be implemented automatically. This is recommended since you won't need to manually
//! implement the size limit logic.
//!
//! To implement the [TryFromChunks](crate::TryFromChunks) trait for external types you will need
//! to create a newtype wrapper and implement the trait for the wrapper.
//!
//! ### Custom error format
//!
//! When using [TypedMultipart](crate::TypedMultipart) as an argument for your handlers, when the
//! request is malformed, the error will be serialized as a string. If you would like to customize
//! the error format you can use the [BaseMultipart](crate::BaseMultipart) struct instead. This
//! struct is used internally by [TypedMultipart](crate::TypedMultipart) and it can be used to
//! customize the error type.
//!
//! To customize the error you will need to define a custom error type and implement
//! [IntoResponse](axum::response::IntoResponse) and `From<TypedMultipartError>`.
//! ```rust,no_run
#![doc = include_str!("../examples/custom_error.rs")]
//! ```
//!
//! ### Validation
//!
//! In order to perform validation on the various attributes of a field, I would recommend using
//! the [validator](https://crates.io/crates/validator) crate together with the
//! [axum-valid](https://crates.io/crates/axum-valid) crate. A nice example can be found at
//! [docs.rs](https://docs.rs/axum-valid/0.19.0/axum_valid/#-validatede-modifiede-validifiede-and-validifiedbyrefe).

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

pub use axum_typed_multipart_macros::{TryFromField, TryFromMultipart};

mod base_multipart;
mod field_data;
mod try_from_chunks;
mod try_from_field;
mod try_from_multipart;
mod typed_multipart;
mod typed_multipart_error;

pub use crate::base_multipart::BaseMultipart;
pub use crate::field_data::{FieldData, FieldMetadata};
pub use crate::try_from_chunks::TryFromChunks;
pub use crate::try_from_field::TryFromField;
pub use crate::try_from_multipart::TryFromMultipart;
pub use crate::typed_multipart::TypedMultipart;
pub use crate::typed_multipart_error::TypedMultipartError;
