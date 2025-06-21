use crate::{FieldMetadata, TryFromChunks, TypedMultipartError};
use async_trait::async_trait;
use axum::body::Bytes;
use futures_core::stream::Stream;

/// Types that can be created from a [Stream] of [Bytes] and a state.
///
/// This is a variant of [TryFromChunks] that allows access to the application
/// state. If you don't need access to the state, consider using that trait
/// instead.
#[async_trait]
pub trait TryFromChunksWithState<S>: Sized {
    /// Consume the input [Stream] of [Bytes] to create the supplied type.
    ///
    /// The `metadata` parameter contains information about the field.
    ///
    /// The `state` parameter contains application state passed from the
    /// [FromRequest](axum::extract::FromRequest) implementation.
    async fn try_from_chunks_with_state(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
        _state: &S,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<T, S> TryFromChunksWithState<S> for T
where
    T: TryFromChunks,
{
    async fn try_from_chunks_with_state(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
        _state: &S,
    ) -> Result<Self, TypedMultipartError> {
        T::try_from_chunks(chunks, metadata).await
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use futures_util::stream;

    struct Data(String);

    #[async_trait]
    impl TryFromChunks for Data {
        async fn try_from_chunks(
            chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
            metadata: FieldMetadata,
        ) -> Result<Self, TypedMultipartError> {
            let string = String::try_from_chunks(chunks, metadata).await?;
            Ok(Data(string))
        }
    }

    #[tokio::test]
    async fn test_try_from_chunks_with_state() {
        let chunks = stream::iter(vec![Ok(Bytes::from("Hello, world!"))]);
        let data = Data::try_from_chunks_with_state(chunks, FieldMetadata::default(), &()).await;
        assert!(matches!(data, Ok(Data(string)) if string == "Hello, world!"));
    }

    #[tokio::test]
    async fn test_try_from_chunks_impl() {
        let chunks = stream::iter(vec![Ok(Bytes::from("Hello, world!"))]);
        let data = Data::try_from_chunks(chunks, FieldMetadata::default()).await;
        assert!(matches!(data, Ok(Data(string)) if string == "Hello, world!"));
    }
}
