use crate::TypedMultipartError;
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// A builder that incrementally consumes multipart fields.
///
/// This trait is primarily used as an implementation detail for [`TryFromMultipart`](crate::TryFromMultipart).
/// For most use cases, derive `TryFromMultipart` instead of using this trait directly.
///
/// The `MultipartBuilder` derive macro generates a builder struct (e.g., `FooBuilder` for `Foo`)
/// that implements this trait. The generated builder is private to the module where the derive
/// is applied.
///
/// Each call to [`consume`](Self::consume) either consumes the field and returns `None`,
/// or returns `Some(field)` unchanged for other builders to handle.
#[async_trait]
pub trait MultipartBuilder<S>: Default {
    /// The type this builder produces.
    type Target;

    /// Attempts to consume a field.
    ///
    /// Returns `Ok(None)` if the field was consumed, or `Ok(Some(field))` if the field
    /// was not recognized and should be passed to another handler.
    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        state: &S,
    ) -> Result<Option<Field<'a>>, TypedMultipartError>;

    /// Finalizes the builder, returning the target or an error if required fields are missing.
    fn finalize(self) -> Result<Self::Target, TypedMultipartError>;
}
