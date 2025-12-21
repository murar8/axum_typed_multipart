use crate::TypedMultipartError;
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// Trait for builder types that incrementally parse multipart form fields.
///
/// This trait is implemented by the derive macro for generated builder structs.
#[async_trait]
pub trait MultipartBuilder<S>: Default + Sized {
    /// The target type this builder constructs.
    type Target;

    /// Process a single multipart field.
    ///
    /// Returns `Ok(None)` if the field was consumed by this builder,
    /// `Ok(Some(field))` if the field was not recognized and should be handled elsewhere.
    async fn process_field<'f>(
        &mut self,
        field_name: &str,
        field: Field<'f>,
        state: &S,
    ) -> Result<Option<Field<'f>>, TypedMultipartError>;

    /// Finalize the builder, returning the constructed type or an error if required fields are missing.
    fn build(self) -> Result<Self::Target, TypedMultipartError>;
}
