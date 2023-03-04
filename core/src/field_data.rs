use crate::field_metadata::FieldMetadata;
use crate::try_from_field::TryFromField;
use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::multipart::Field;

/// Wrapper struct that allows to retrieve both the field contents and the
/// additional metadata provided by the client.
///
/// This is mainly useful for file uploads but can be used for every field where
/// you need access to the metadata.
///
/// If the generic argument implements
/// [TryFromField](crate::try_from_field::TryFromField) the struct will
/// implement the trait itself.
///
/// ## Example
///
/// ```rust
/// use axum_typed_multipart::{FieldData, TryFromMultipart};
///
/// #[derive(TryFromMultipart)]
/// struct FileUpload {
///     input_file: FieldData<String>,
/// }
/// ```
pub struct FieldData<T> {
    pub metadata: FieldMetadata,
    pub contents: T,
}

#[async_trait]
impl<T: TryFromField> TryFromField for FieldData<T> {
    async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError> {
        let metadata = FieldMetadata::from(&field);
        let contents = T::try_from_field(field).await?;
        Ok(Self { metadata, contents })
    }
}
