use crate::{TryFromMultipart, TypedMultipartError};
use async_trait::async_trait;
use axum::extract::Multipart;

#[async_trait]
pub trait StatefulTryFromMultipart<S>: Sized {
    async fn try_from_multipart_with_state(
        multipart: &mut Multipart,
        state: &S,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<T, S> StatefulTryFromMultipart<S> for T
where
    T: TryFromMultipart,
{
    async fn try_from_multipart_with_state(
        multipart: &mut Multipart,
        _: &S,
    ) -> Result<Self, TypedMultipartError> {
        T::try_from_multipart(multipart).await
    }
}
