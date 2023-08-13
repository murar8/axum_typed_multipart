use crate::{TryFromMultipart, TypedMultipartError};
use axum::body::{Bytes, HttpBody};
use axum::extract::{FromRequest, Multipart};
use axum::http::Request;
use axum::response::IntoResponse;
use axum::{async_trait, BoxError};
use std::marker::PhantomData;

/// Used as as an argument for axum [Handlers](axum::handler::Handler).
///
/// Implements [FromRequest] when the generic argument implements the
/// [TryFromMultipart] trait and the generic rejection implements the
/// [IntoResponse] and `From<TypedMultipartError>` traits.
///
/// ## Example
///
/// ```rust
/// use axum::http::StatusCode;
/// use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
///
/// #[derive(TryFromMultipart)]
/// struct Data {
///     name: String,
///     email: Option<String>,
///     url: Option<String>,
/// }
///
/// type CustomMultipart<T> = BaseMultipart<T, TypedMultipartError>;
///
/// async fn handle_data(CustomMultipart { data, .. }: CustomMultipart<Data>) -> StatusCode {
///     println!("name: {}", data.name);
///     println!("email: {}", data.email.unwrap_or_default());
///     println!("url: {}", data.url.unwrap_or_default());
///     StatusCode::OK
/// }
/// ```
pub struct BaseMultipart<T, R> {
    pub data: T,
    rejection: PhantomData<R>,
}

#[async_trait]
impl<S, B, T, R> FromRequest<S, B> for BaseMultipart<T, R>
where
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<BoxError>,
    T: TryFromMultipart,
    R: IntoResponse + From<TypedMultipartError>,
{
    type Rejection = R;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let multipart = &mut Multipart::from_request(req, state).await.map_err(Into::into)?;
        let data = T::try_from_multipart(multipart).await?;
        Ok(Self { data, rejection: PhantomData })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Multipart;
    use axum::routing::post;
    use axum::{async_trait, Router};
    use axum_test_helper::TestClient;
    use reqwest::multipart::Form;

    struct Data(String);

    #[async_trait]
    impl TryFromMultipart for Data {
        async fn try_from_multipart(_: &mut Multipart) -> Result<Self, TypedMultipartError> {
            Ok(Self(String::from("data")))
        }
    }

    #[tokio::test]
    async fn test_typed_multipart() {
        async fn handler(BaseMultipart { data, .. }: BaseMultipart<Data, TypedMultipartError>) {
            assert_eq!(data.0, "data");
        }

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new())
            .send()
            .await;
    }
}
