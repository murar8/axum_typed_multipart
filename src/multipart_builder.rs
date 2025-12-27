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
    /// The `name` parameter is the effective field name to match against. For top-level
    /// fields this is `field.name()`, for nested fields it's the name with prefix stripped.
    ///
    /// Returns `Ok(None)` if the field was consumed, or `Ok(Some(field))` if the field
    /// was not recognized and should be passed to another handler.
    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        name: Option<&str>,
        state: &S,
    ) -> Result<Option<Field<'a>>, TypedMultipartError>;

    /// Finalizes the builder, returning the target or an error if required fields are missing.
    fn finalize(self) -> Result<Self::Target, TypedMultipartError>;
}

/// Blanket impl for `Vec<B>` - parses indexed field names like `[0].field`.
#[async_trait]
impl<S, B> MultipartBuilder<S> for Vec<B>
where
    S: Sync,
    B: MultipartBuilder<S> + Send,
{
    type Target = Vec<B::Target>;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        name: Option<&str>,
        state: &S,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        let Some(name) = name.filter(|n| !n.is_empty()) else {
            return Ok(Some(field));
        };
        if let Some(rest) = name.strip_prefix('[') {
            if let Some(end) = rest.find(']') {
                if let Ok(idx) = rest[..end].parse::<usize>() {
                    let rest = &rest[end + 1..];
                    let rest = rest.strip_prefix('.').unwrap_or(rest);
                    if self.len() <= idx {
                        self.resize_with(idx + 1, Default::default);
                    }
                    return self[idx].consume(field, Some(rest), state).await;
                }
            }
        }
        Ok(Some(field))
    }

    fn finalize(self) -> Result<Self::Target, TypedMultipartError> {
        self.into_iter().map(MultipartBuilder::finalize).collect()
    }
}

/// Blanket impl for `Option<B>` - creates inner builder on first field.
#[async_trait]
impl<S, B> MultipartBuilder<S> for Option<B>
where
    S: Sync,
    B: MultipartBuilder<S> + Send,
{
    type Target = Option<B::Target>;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        name: Option<&str>,
        state: &S,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        self.get_or_insert_with(Default::default).consume(field, name, state).await
    }

    fn finalize(self) -> Result<Self::Target, TypedMultipartError> {
        self.map(MultipartBuilder::finalize).transpose()
    }
}
