use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::multipart::Field;

#[async_trait]
pub trait TryFromField<'a>: Sized {
    async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<'a> TryFromField<'a> for String {
    async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError> {
        let text = field.text().await?;
        Ok(text)
    }
}
