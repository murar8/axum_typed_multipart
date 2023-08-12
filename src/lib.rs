//! Designed to seamlessly integrate with
//! [Axum](https://github.com/tokio-rs/axum), this crate simplifies the process
//! of handling `multipart/form-data` requests in your web application by
//! allowing you to parse the request body into a type-safe struct.
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
//!
//! To be able to derive the [TryFromMultipart](crate::TryFromMultipart) trait
//! every field in the struct must implement the
//! [TryFromField](crate::TryFromField) trait. The trait is implemented by
//! default for all primitive types, [String], [Bytes](axum::body::Bytes), and
//! [NamedTempFile](tempfile::NamedTempFile).
//!
//! If the request body is malformed or it does not contain the required data
//! the request will be aborted with an error.
//!
//! ```no_run
//! use axum::http::StatusCode;
//! use axum::routing::post;
//! use axum::Router;
//! use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
//! use std::net::SocketAddr;
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     first_name: String,
//!     last_name: String,
//! }
//!
//! async fn handler(
//!     TypedMultipart(RequestData { first_name, last_name }): TypedMultipart<RequestData>,
//! ) -> StatusCode {
//!     println!("full name = '{} {}'", first_name, last_name);
//!     StatusCode::OK
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
//!         .serve(Router::new().route("/", post(handler)).into_make_service())
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! ### Optional fields
//!
//! If a field is declared as an [Option] the value will default to [None] when
//! the field is missing from the request body.
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
//! If you would like to assign a custom name for the source field you can use
//! the `field_name` parameter of the `form_data` attribute.
//!
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
//! ### Default values
//!
//! If the `default` parameter in the `form_data` attribute is present the value
//! will be populated using the type's [Default] implementation when the field
//! is not supplied in the request.
//!
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
//! If you need access to the field metadata (e.g. the request headers) you can
//! use the [FieldData](crate::FieldData) struct to wrap your field.
//!
//! ```rust
//! use axum::body::Bytes;
//! use axum::http::StatusCode;
//! use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     image: FieldData<Bytes>,
//! }
//!
//! async fn handler(
//!     TypedMultipart(RequestData { image }): TypedMultipart<RequestData>,
//! ) -> StatusCode {
//!     println!("file name = '{}'", image.metadata.file_name.unwrap());
//!     println!("content type = '{}'", image.metadata.content_type.unwrap());
//!     println!("size = {}b", image.contents.len());
//!     StatusCode::OK
//! }
//! ```
//!
//! ### Large uploads
//!
//! For large uploads you can save the contents of the field to the file system
//! using the [NamedTempFile](tempfile::NamedTempFile) crate. This will
//! efficiently stream the field data directly to the file system, without
//! needing to fit all the data in memory. Once the upload is complete, you can
//! then save the contents to a location of your choice. For more information
//! check out the [NamedTempFile](tempfile::NamedTempFile) documentation.
//!
//! #### **Warning**
//! Field size limits for [Vec] fields are applied to **each** occurrence of the
//! field. This means that if you have a 1GiB field limit and the field contains
//! 5 entries, the total size of the request body will be 5GiB.
//!
//! #### **Note**
//! When handling large uploads you will need to increase both the request body
//! size limit and the field size limit. The request body size limit can be
//! increased using the [DefaultBodyLimit](axum::extract::DefaultBodyLimit)
//! middleware, while the field size limit can be increased using the `limit`
//! parameter of the `form_data` attribute.
//!
//! ```no_run
//! use axum::extract::DefaultBodyLimit;
//! use axum::http::StatusCode;
//! use axum::routing::post;
//! use axum::Router;
//! use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
//! use std::net::SocketAddr;
//! use std::path::Path;
//! use tempfile::NamedTempFile;
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     #[form_data(limit = "unlimited")]
//!     image: FieldData<NamedTempFile>, // This field will be limited to the size of the request body.
//!
//!     author: String, // This field will be limited to the default size of 1MiB.
//! }
//!
//! async fn handler(
//!     TypedMultipart(RequestData { image, author }): TypedMultipart<RequestData>,
//! ) -> StatusCode {
//!     let file_name = image.metadata.file_name.unwrap_or(String::from("data.bin"));
//!     let path = Path::new("/tmp").join(author).join(file_name);
//!
//!     match image.contents.persist(path) {
//!         Ok(_) => StatusCode::OK,
//!         Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = Router::new()
//!         .route("/", post(handler))
//!         // The default axum body size limit is 2MiB, so we need to increase
//!         // it to 1GiB.
//!         .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));
//!
//!     axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
//!         .serve(router.into_make_service())
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! ### Lists
//!
//! If the incoming request will include multiple fields that share the same
//! name (AKA lists) the field can be declared as a [Vec], allowing for all
//! occurrences of the field to be stored.
//!
//! #### **Warning**
//! Field size limits for [Vec] fields are applied to **each** occurrence of the
//! field. This means that if you have a 1GiB field limit and the field contains
//! 5 entries, the total size of the request body will be 5GiB.
//!
//! ```rust
//! use axum::http::StatusCode;
//! use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
//!
//! #[derive(TryFromMultipart)]
//! struct RequestData {
//!     names: Vec<String>,
//! }
//!
//! async fn handler(
//!     TypedMultipart(RequestData { names }): TypedMultipart<RequestData>,
//! ) -> StatusCode {
//!     println!("first name = '{}'", names[0]);
//!     StatusCode::OK
//! }
//! ```
//!
//! ### Strict mode
//!
//! By default the derive macro will store the last occurrence of a field and it
//! will ignore unknown fields. This behavior can be changed by using the
//! `strict` parameter in the derive macro. This will make the macro throw an
//! error if the request contains multiple fields with the same name or if it
//! contains unknown fields.
//!
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
//! ### Custom types
//!
//! If you would like to use a custom type for a field you need to implement the
//! [TryFromField](crate::TryFromField) trait for your type. This will allow the
//! derive macro to generate the [TryFromMultipart](crate::TryFromMultipart)
//! implementation automatically. Instead of implementing the trait directly, it
//! is recommended to implement the [TryFromChunks](crate::TryFromChunks) trait
//! and the [TryFromField](crate::TryFromField) trait will be implemented
//! automatically. This is recommended since you won't need to manually
//! implement the size limit logic.
//!
//! If you implement the trait for a common type for some external crate, feel
//! free to submit a PR to add it to the crate!

pub use axum_typed_multipart_macros::TryFromMultipart;

mod field_data;
mod try_from_chunks;
mod try_from_field;
mod try_from_multipart;
mod typed_multipart;
mod typed_multipart_error;

pub use crate::field_data::{FieldData, FieldMetadata};
pub use crate::try_from_chunks::TryFromChunks;
pub use crate::try_from_field::TryFromField;
pub use crate::try_from_multipart::TryFromMultipart;
pub use crate::typed_multipart::TypedMultipart;
pub use crate::typed_multipart_error::TypedMultipartError;
