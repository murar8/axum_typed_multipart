use crate::{TryFromMultipart, TypedMultipartError};
use axum::async_trait;
use axum::extract::{FromRequest, Multipart, Request};
use axum::response::IntoResponse;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Used as an argument for axum [Handlers](axum::handler::Handler).
///
/// Implements [FromRequest] when the generic argument implements the
/// [TryFromMultipart] trait and the generic rejection implements the
/// [IntoResponse] and `From<TypedMultipartError>` traits.
///
/// ## Example
///
/// ```rust
/// use axum::http::StatusCode;
/// use axum_typed_multipart::{BaseMultipart, TryFromMultipart, TypedMultipartError};
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
/// async fn handle_data(data: CustomMultipart<Data>) -> StatusCode {
///     println!("name: {}", data.name);
///     println!("email: {}", data.email.as_deref().unwrap_or_default());
///     println!("url: {}", data.url.as_deref().unwrap_or_default());
///     StatusCode::OK
/// }
/// ```
pub struct BaseMultipart<T, R> {
    pub data: T,
    rejection: PhantomData<R>,
}

impl<T, R> Deref for BaseMultipart<T, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, R> DerefMut for BaseMultipart<T, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[async_trait]
impl<S, T, R> FromRequest<S> for BaseMultipart<T, R>
where
    S: Send + Sync,
    T: TryFromMultipart,
    R: IntoResponse + From<TypedMultipartError>,
{
    type Rejection = R;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let multipart = &mut Multipart::from_request(req, state).await.map_err(Into::into)?;
        let data = T::try_from_multipart(multipart).await?;
        Ok(Self { data, rejection: PhantomData })
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
            .await
            .post("/")
            .multipart(Form::new())
            .send()
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
        let mut tm = BaseMultipart { data: data.clone(), rejection: PhantomData::<()> };
        assert_eq!(tm.deref(), &data);

        data.modify();
        tm.modify();
        assert_eq!(tm.deref(), &data);
    }
}
