use crate::try_from_multipart::TryFromMultipart;
use crate::typed_multipart_error::TypedMultipartError;
use axum::body::{Bytes, HttpBody};
use axum::extract::{FromRequest, Multipart};
use axum::http::Request;
use axum::{async_trait, BoxError};

#[derive(Debug)]
pub struct TypedMultipart<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for TypedMultipart<T>
where
    T: TryFromMultipart,
    B: HttpBody + Send + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = TypedMultipartError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let multipart = Multipart::from_request(req, state).await?;
        let data = T::try_from_multipart(multipart).await?;
        Ok(Self(data))
    }
}
