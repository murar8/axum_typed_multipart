use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::Multipart;

/// Types that can be created from an instance of [Multipart].
///
/// Structs that implement this trait can be used as type parameters for
/// [TypedMultipart] allowing to generate the supplied struct from the request.
///
/// The trait can be implemented using the corresponding derive macro.
#[async_trait]
pub trait TryFromMultipart: Sized {
    async fn try_from_multipart(multipart: Multipart) -> Result<Self, TypedMultipartError>;
}
