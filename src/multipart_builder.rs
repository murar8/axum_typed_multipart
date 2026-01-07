use crate::parse_index::parse_index;
use crate::TypedMultipartError;
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// A builder that incrementally consumes multipart fields.
///
/// This trait is primarily used as an implementation detail for [`TryFromMultipart`](crate::TryFromMultipart).
/// For most use cases, derive `TryFromMultipart` instead of using this trait directly.
///
/// The `MultipartBuilder` derive macro generates a builder struct (e.g., `FooMultipartBuilder` for `Foo`)
/// that implements this trait. The generated builder has the same visibility as the original struct
/// (but is hidden from rustdoc with `#[doc(hidden)]`).
///
/// Each call to [`consume`](Self::consume) either consumes the field and returns `None`,
/// or returns `Some(field)` unchanged for other builders to handle.
#[async_trait]
pub trait MultipartBuilder<S> {
    /// The type this builder produces.
    type Target;

    /// Attempts to consume a field.
    ///
    /// The `suffix` parameter contains the remaining unparsed portion of the field name.
    /// For top-level fields, this is the full name (e.g., `"users[0].name"`).
    /// For nested fields, this is the suffix after the parent's prefix (e.g., `"[0].name"` or `".name"`).
    ///
    /// The `depth` parameter indicates the current nesting depth (0 for top-level).
    ///
    /// Returns `Ok(None)` if the field was consumed, or `Ok(Some(field))` if the field
    /// was not recognized and should be passed to another handler.
    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        suffix: &str,
        state: &S,
        depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError>;

    /// Finalizes the builder, returning the target or an error if required fields are missing.
    ///
    /// The `path` parameter contains the field path prefix for error messages (e.g., "person.address").
    /// For top-level builders, pass an empty string. Nested builders receive the accumulated path.
    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError>;

    /// Returns `true` if any field was consumed by this builder or its nested builders.
    ///
    /// Used to detect when an entire required nested struct is missing, allowing error messages
    /// to report the parent field (e.g., "field 'owner' is required") instead of the first
    /// missing leaf field (e.g., "field 'owner.name' is required").
    fn was_consumed(&self) -> bool;
}

/// Blanket impl for `Vec<B>` - parses indexed field names like `[0].field`.
///
/// This impl is used by macro-generated builder structs for `#[form_data(nested)]` fields
/// with `Vec<T>` types. The macro generates `Vec<TMultipartBuilder>` fields, which use this
/// impl to parse indexed field names and delegate to the inner builder.
///
/// Indices must be consecutive starting from 0 and fields must arrive in index order.
/// For example, `[0].name`, `[0].age`, `[1].name`, `[1].age` is valid, but `[1].name` before
/// any `[0]` field would be rejected. Fields for the same index can arrive in any order.
///
/// Invalid index formats (e.g., `[abc]`, `[-1]`, missing `]`) return an error.
#[async_trait]
impl<S, B> MultipartBuilder<S> for Vec<B>
where
    S: Sync,
    B: MultipartBuilder<S> + Send + Default,
{
    type Target = Vec<B::Target>;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        suffix: &str,
        state: &S,
        depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        let field_name = || field.name().unwrap_or_default().to_string();
        let (idx, rest) = parse_index(suffix).map_err(|source| {
            TypedMultipartError::InvalidIndexFormat { field_name: field_name(), source }
        })?;

        if idx == self.len() {
            // Next consecutive index - create new builder
            self.push(B::default());
        }
        if idx < self.len() {
            // Valid index - delegate to inner builder
            self[idx].consume(field, rest, state, depth + 1).await
        } else {
            // Gap in indices (idx > self.len()) - error
            Err(TypedMultipartError::InvalidIndex {
                field_name: field_name(),
                expected: self.len(),
            })
        }
    }

    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError> {
        self.into_iter()
            .enumerate()
            .map(|(idx, builder)| builder.finalize(&format!("{path}[{idx}]")))
            .collect()
    }

    fn was_consumed(&self) -> bool {
        !self.is_empty()
    }
}

/// Blanket impl for `Option<B>` - creates inner builder on first field.
///
/// This impl is used by macro-generated builder structs for `#[form_data(nested)]` fields
/// with `Option<T>` types. The macro generates `Option<TMultipartBuilder>` fields, which
/// use this impl to lazily initialize the inner builder when the first field arrives.
#[async_trait]
impl<S, B> MultipartBuilder<S> for Option<B>
where
    S: Sync,
    B: MultipartBuilder<S> + Send + Default,
{
    type Target = Option<B::Target>;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        suffix: &str,
        state: &S,
        depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        self.get_or_insert_default().consume(field, suffix, state, depth).await
    }

    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError> {
        self.map(|b| b.finalize(path)).transpose()
    }

    fn was_consumed(&self) -> bool {
        self.is_some()
    }
}
