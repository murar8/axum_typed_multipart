use crate::TypedMultipartError;
use async_trait::async_trait;
use axum::extract::Multipart;

/// Types that can be created from an instance of [Multipart].
///
/// Structs that implement this trait can be used as type parameters for
/// [TypedMultipart](crate::TypedMultipart) allowing to generate the supplied
/// struct from the request data.
///
/// ## Example
///
/// ```rust
/// use axum_typed_multipart::TryFromMultipart;
///
/// #[derive(TryFromMultipart)]
/// struct Data {
///     name: String,
/// }
/// ```
#[async_trait]
pub trait TryFromMultipart: Sized {
    async fn try_from_multipart(multipart: &mut Multipart) -> Result<Self, TypedMultipartError>;
}
