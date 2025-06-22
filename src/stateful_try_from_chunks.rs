use crate::{FieldMetadata, TryFromChunks, TypedMultipartError};
use async_trait::async_trait;
use axum::body::Bytes;
use futures_core::stream::Stream;

/// Types that can be created from a [Stream] of [Bytes] with a state.
#[async_trait]
pub trait StatefulTryFromChunks<S>: Sized {
    async fn try_from_chunks_with_state(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
        state: &S,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<T, S> StatefulTryFromChunks<S> for T
where
    T: TryFromChunks,
{
    async fn try_from_chunks_with_state(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
        _: &S,
    ) -> Result<Self, TypedMultipartError> {
        Self::try_from_chunks(chunks, metadata).await
    }
}
