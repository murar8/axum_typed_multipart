use crate::{BaseMultipart, TryFromMultipart, TypedMultipartError};
use axum::body::{Bytes, HttpBody};
use axum::extract::FromRequest;
use axum::http::Request;
use axum::{async_trait, BoxError};

/// Used as as an argument for axum [Handlers](axum::handler::Handler).
///
/// Implements [FromRequest] when the generic argument implements the
/// [TryFromMultipart] trait. If you need to `extract` the TypedMultipart data inside a FromRequest
/// query, you need to provide additional trait bounds, as you can see in `examples/from_request.rs`.
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
/// async fn handle_data(TypedMultipart(data): TypedMultipart<Data>) -> StatusCode {
///     println!("name: {}", data.name);
///     println!("email: {}", data.email.unwrap_or_default());
///     println!("url: {}", data.url.unwrap_or_default());
///     StatusCode::OK
/// }
/// ```
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
        let base = BaseMultipart::<T, Self::Rejection>::from_request(req, state).await?;
        Ok(Self(base.data))
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
        async fn handler(TypedMultipart(data): TypedMultipart<Data>) {
            assert_eq!(data.0, "data");
        }

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new())
            .send()
            .await;
    }
}
