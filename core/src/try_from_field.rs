use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::multipart::Field;

/// Types that can be crated from an instance of [Field].
///
/// All fields for a given struct must implement this trait to be able to derive
/// the [TryFromMultipart] trait.
///
/// ### Example
///
/// ```rust
/// use axum::async_trait;
/// use axum::extract::multipart::Field;
/// use axum_typed_multipart::{TryFromField, TypedMultipartError};
///
/// struct CoolString(String);
///
/// #[async_trait]
/// impl<'a> TryFromField<'a> for CoolString {
///     async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError> {
///         let text = field.text().await?;
///         Ok(CoolString(text))
///     }
/// }
/// ```
#[async_trait]
pub trait TryFromField<'a>: Sized {
    /// Consume the input [Field] to create the supplied type.
    async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<'a> TryFromField<'a> for String {
    async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError> {
        let text = field.text().await?;
        Ok(text)
    }
}
