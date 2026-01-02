//! Test crate for cross-crate nested type support.

use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart, Debug, PartialEq)]
pub struct ExternalAddress {
    pub street: String,
    pub city: String,
}
