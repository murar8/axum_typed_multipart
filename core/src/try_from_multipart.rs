use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::Multipart;

#[async_trait]
pub trait TryFromMultipart: Sized {
    async fn try_from_multipart(multipart: Multipart) -> Result<Self, TypedMultipartError>;
}
