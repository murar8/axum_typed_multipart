use crate::TypedMultipartError;
use async_trait::async_trait;
use axum::extract::multipart::Field;
use serde_spanned::Spanned;
use std::collections::BTreeMap;

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
    /// The `name` parameter contains the full field name and a span indicating the current
    /// segment to match. For top-level fields, the span covers the entire name. For nested
    /// fields, the span covers only the suffix after the parent's prefix.
    ///
    /// The `depth` parameter indicates the current nesting depth (0 for top-level).
    ///
    /// Returns `Ok(None)` if the field was consumed, or `Ok(Some(field))` if the field
    /// was not recognized and should be passed to another handler.
    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        name: Spanned<&str>,
        state: &S,
        depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError>;

    /// Finalizes the builder, returning the target or an error if required fields are missing.
    ///
    /// The `path` parameter contains the field path prefix for error messages (e.g., "person.address").
    /// For top-level builders, pass an empty string. Nested builders receive the accumulated path.
    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError>;
}

/// Parses `[index]` from the start of the current span.
/// Returns the index and a new `Spanned` with span advanced past `]`.
fn parse_index<'a>(name: &Spanned<&'a str>) -> Option<(usize, Spanned<&'a str>)> {
    let full = *name.as_ref();
    let span = name.span();

    let rest = full[span.start..span.end].strip_prefix('[')?;
    let end = rest.find(']')?;
    let idx = rest[..end].parse::<usize>().ok()?;

    Some((idx, Spanned::new(span.start + 1 + end + 1..span.end, full)))
}

/// Blanket impl for `BTreeMap<usize, B>` - parses indexed field names like `[0].field`.
///
/// This impl is used by macro-generated builder structs for `#[form_data(nested)]` fields
/// with `Vec<T>` types. The macro generates `BTreeMap<usize, TMultipartBuilder>` fields,
/// which use this impl to parse indexed field names and delegate to the inner builder.
///
/// Uses a map instead of a vector to support sparse indices and prevent DoS via large indices.
/// A `Vec` would allocate memory proportional to the largest index (e.g., `[999999999]` would
/// allocate a billion slots). With `BTreeMap`, memory is proportional to the number of entries.
/// Since each multipart field has overhead (~80+ bytes for boundary and headers), Axum's body
/// size limits naturally bound the number of entries that can be created.
#[async_trait]
impl<S, B> MultipartBuilder<S> for BTreeMap<usize, B>
where
    S: Sync,
    B: MultipartBuilder<S> + Send,
{
    type Target = Vec<B::Target>;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        name: Spanned<&str>,
        state: &S,
        depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        match parse_index(&name) {
            None => Ok(Some(field)), // No index - cannot consume
            Some((idx, rest)) => {
                self.entry(idx).or_default().consume(field, rest, state, depth + 1).await
            }
        }
    }

    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError> {
        self.into_iter().map(|(idx, builder)| builder.finalize(&format!("{path}[{idx}]"))).collect()
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
    B: MultipartBuilder<S> + Send,
{
    type Target = Option<B::Target>;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        name: Spanned<&str>,
        state: &S,
        depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        self.get_or_insert_with(Default::default).consume(field, name, state, depth).await
    }

    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError> {
        self.map(|b| b.finalize(path)).transpose()
    }
}
