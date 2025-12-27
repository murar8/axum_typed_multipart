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

/// Parses `[index]` from the start of name.
/// Returns the index and remainder after `]`.
fn parse_index(name: &str) -> Option<(usize, &str)> {
    let rest = name.strip_prefix('[')?;
    let end = rest.find(']')?;
    let idx = rest[..end].parse::<usize>().ok()?;
    Some((idx, &rest[end + 1..]))
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
        let (idx, rest) = match name.and_then(parse_index) {
            Some(v) => v,
            None => return Ok(Some(field)), // No index - cannot consume
        };
        self.resize_with(idx + 1, Default::default);
        self[idx].consume(field, Some(rest), state).await
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
