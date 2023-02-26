use crate::field_metadata::FieldMetadata;

/// Wrapper struct that allows to retrieve both the field contents and the
/// additional information uploaded in the form.
///
/// If the generic argument implements
/// [TryFromField](axum_typed_multipart::try_from_field::TryFromField) the
/// struct will implement the trait itself.
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
