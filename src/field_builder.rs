use crate::field_name::Segment;
use crate::{TryFromFieldWithState, TypedMultipartError};
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// Trait for types that accumulate multipart field occurrences and produce a
/// final value.
///
/// The derive macro for [`TryFromMultipart`](crate::TryFromMultipart) selects a
/// concrete builder for each struct field based on its type:
///
/// | Field type   | Builder                      |
/// |--------------|------------------------------|
/// | `T`          | [`RequiredBuilder<T>`]       |
/// | `Option<T>`  | `Option<RequiredBuilder<T>>` |
/// | `Vec<T>`     | `Vec<RequiredBuilder<T>>`    |
#[async_trait]
pub trait FieldBuilder<S: Sync>: Default {
    /// The type produced after all fields have been consumed.
    type Output;

    /// Check if this builder accepts the given field name.
    ///
    /// `segments` is the parsed field name from the incoming request.
    /// `base_name` is the name of the struct field this builder is responsible
    /// for.
    fn matches(segments: &[Segment<'_>], base_name: &str) -> bool;

    /// Process a single field occurrence. The caller must verify
    /// [`matches`](Self::matches) returns `true` before calling this.
    ///
    /// `segments` is the same parsed field name that was passed to
    /// [`matches`](Self::matches). Leaf builders can ignore it; nested
    /// builders use it to dispatch to inner fields. `field_name` is
    /// the original unparsed field name for error messages.
    async fn push_field(
        &mut self,
        field: Field<'_>,
        segments: &[Segment<'_>],
        field_name: &str,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<(), TypedMultipartError>;

    /// Consume the builder and produce the final value.
    ///
    /// `field_name` is provided for error messages.
    fn finalize(self, field_name: &str) -> Result<Self::Output, TypedMultipartError>;

    /// Returns `true` if at least one value has been accumulated.
    fn has_value(&self) -> bool;

    /// Whether this builder accepts multiple occurrences of the same field.
    fn allows_multiple(&self) -> bool;
}

/// Builder for required fields. Returns [`TypedMultipartError::MissingField`]
/// if no value was provided.
pub struct RequiredBuilder<T>(Option<T>);

impl<T> Default for RequiredBuilder<T> {
    fn default() -> Self {
        Self(None)
    }
}

#[async_trait]
impl<T, S> FieldBuilder<S> for RequiredBuilder<T>
where
    T: TryFromFieldWithState<S> + Send,
    S: Sync,
{
    type Output = T;

    fn matches(segments: &[Segment<'_>], base_name: &str) -> bool {
        matches!(segments, [Segment::Key(k)] if *k == base_name)
    }

    async fn push_field(
        &mut self,
        field: Field<'_>,
        segments: &[Segment<'_>],
        _field_name: &str,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<(), TypedMultipartError> {
        debug_assert!(matches!(segments, [Segment::Key(_)]));
        self.0 = Some(T::try_from_field_with_state(field, limit_bytes, state).await?);
        Ok(())
    }

    fn finalize(self, field_name: &str) -> Result<Self::Output, TypedMultipartError> {
        self.0
            .ok_or_else(|| TypedMultipartError::MissingField { field_name: field_name.to_owned() })
    }

    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn allows_multiple(&self) -> bool {
        false
    }
}

/// Builder for fields with `#[form_data(default)]`. Uses [`Default::default()`]
/// when no value was provided.
pub struct DefaultBuilder<T>(Option<T>);

impl<T> Default for DefaultBuilder<T> {
    fn default() -> Self {
        Self(None)
    }
}

#[async_trait]
impl<T, S> FieldBuilder<S> for DefaultBuilder<T>
where
    T: TryFromFieldWithState<S> + Default + Send,
    S: Sync,
{
    type Output = T;

    fn matches(segments: &[Segment<'_>], base_name: &str) -> bool {
        matches!(segments, [Segment::Key(k)] if *k == base_name)
    }

    async fn push_field(
        &mut self,
        field: Field<'_>,
        segments: &[Segment<'_>],
        _field_name: &str,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<(), TypedMultipartError> {
        debug_assert!(matches!(segments, [Segment::Key(_)]));
        self.0 = Some(T::try_from_field_with_state(field, limit_bytes, state).await?);
        Ok(())
    }

    fn finalize(self, _field_name: &str) -> Result<Self::Output, TypedMultipartError> {
        Ok(self.0.unwrap_or_default())
    }

    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn allows_multiple(&self) -> bool {
        false
    }
}

#[async_trait]
impl<T, S> FieldBuilder<S> for Option<T>
where
    T: FieldBuilder<S> + Send,
    S: Sync,
{
    type Output = Option<T::Output>;

    fn matches(segments: &[Segment<'_>], base_name: &str) -> bool {
        T::matches(segments, base_name)
    }

    async fn push_field(
        &mut self,
        field: Field<'_>,
        segments: &[Segment<'_>],
        field_name: &str,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<(), TypedMultipartError> {
        self.get_or_insert_with(T::default)
            .push_field(field, segments, field_name, limit_bytes, state)
            .await
    }

    fn finalize(self, field_name: &str) -> Result<Self::Output, TypedMultipartError> {
        self.map(|inner| inner.finalize(field_name)).transpose()
    }

    fn has_value(&self) -> bool {
        self.is_some()
    }

    fn allows_multiple(&self) -> bool {
        self.as_ref().is_some_and(|inner| inner.allows_multiple())
    }
}

#[async_trait]
impl<T, S> FieldBuilder<S> for Vec<T>
where
    T: FieldBuilder<S> + Send,
    S: Sync,
{
    type Output = Vec<T::Output>;

    fn matches(segments: &[Segment<'_>], base_name: &str) -> bool {
        T::matches(segments, base_name)
    }

    async fn push_field(
        &mut self,
        field: Field<'_>,
        segments: &[Segment<'_>],
        field_name: &str,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<(), TypedMultipartError> {
        let mut builder = T::default();
        builder.push_field(field, segments, field_name, limit_bytes, state).await?;
        self.push(builder);
        Ok(())
    }

    fn finalize(self, field_name: &str) -> Result<Self::Output, TypedMultipartError> {
        self.into_iter().map(|b| b.finalize(field_name)).collect()
    }

    fn has_value(&self) -> bool {
        !self.is_empty()
    }

    fn allows_multiple(&self) -> bool {
        true
    }
}
