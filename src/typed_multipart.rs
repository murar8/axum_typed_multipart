use crate::{BaseMultipart, TryFromMultipartWithState, TypedMultipartError};
use axum::extract::{FromRequest, Request};
use std::ops::{Deref, DerefMut};

/// Used as an argument for axum [Handlers](axum::handler::Handler).
///
/// Implements [FromRequest] when the generic argument implements the
/// [TryFromMultipart](crate::TryFromMultipart) trait.
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
/// async fn handle_data(data: TypedMultipart<Data>) -> StatusCode {
///     println!("name: {}", data.name);
///     println!("email: {}", data.email.as_deref().unwrap_or_default());
///     println!("url: {}", data.url.as_deref().unwrap_or_default());
///     StatusCode::OK
/// }
/// ```
#[derive(Debug)]
pub struct TypedMultipart<T>(pub T);

impl<T> Deref for TypedMultipart<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for TypedMultipart<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, S> FromRequest<S> for TypedMultipart<T>
where
    T: TryFromMultipartWithState<S>,
    S: Send + Sync,
{
    type Rejection = TypedMultipartError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let base = BaseMultipart::<T, Self::Rejection>::from_request(req, state).await?;
        Ok(Self(base.data))
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use crate::TryFromMultipart;
    use axum::extract::Multipart;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::Form;

    struct Data(String);

    #[async_trait::async_trait]
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
            .await;
    }

    #[test]
    fn test_deref() {
        #[derive(Debug, Clone, Eq, PartialEq)]
        struct Data {
            v0: String,
            v1: i32,
        }

        impl Data {
            fn modify(&mut self) {
                self.v0.push_str("modified");
                self.v1 += 1;
            }
        }

        let mut data = Data { v0: "DATA".into(), v1: 12 };
        let mut tm = TypedMultipart(data.clone());
        assert_eq!(tm.deref(), &data);

        data.modify();
        tm.modify();
        assert_eq!(tm.deref(), &data);
    }
}
