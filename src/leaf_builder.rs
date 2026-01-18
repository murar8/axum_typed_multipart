//! Builder wrapper for leaf (non-nested) field types.

use crate::{MultipartBuilder, TryFromFieldWithState, TypedMultipartError};
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// Builder wrapper for leaf field types that implement `TryFromFieldWithState`.
///
/// This wrapper allows primitive types like `String`, `i32`, etc. to be used inside
/// `Vec<T>` and `Option<T>` through the `MultipartBuilder` trait, enabling
/// uniform handling of all field types.
///
/// The wrapper is transparent: `LeafBuilder<T>` produces `T` when finalized.
#[derive(Debug)]
pub struct LeafBuilder<T>(Option<T>);

impl<T> Default for LeafBuilder<T> {
    fn default() -> Self {
        Self(None)
    }
}

#[async_trait]
impl<S, T> MultipartBuilder<S> for LeafBuilder<T>
where
    S: Sync,
    T: TryFromFieldWithState<S> + Send,
{
    type Target = T;

    async fn consume<'a>(
        &mut self,
        field: Field<'a>,
        suffix: &str,
        state: &S,
        limit_bytes: Option<usize>,
        _depth: usize,
    ) -> Result<Option<Field<'a>>, TypedMultipartError> {
        // Leaf fields only accept empty suffix (exact match)
        if suffix.is_empty() {
            self.0 = T::try_from_field_with_state(field, limit_bytes, state).await?.into();
            Ok(None)
        } else {
            Ok(Some(field))
        }
    }

    fn finalize(self, path: &str) -> Result<Self::Target, TypedMultipartError> {
        self.0.ok_or_else(|| TypedMultipartError::MissingField { field_name: path.to_string() })
    }

    fn was_consumed(&self) -> bool {
        self.0.is_some()
    }
}
