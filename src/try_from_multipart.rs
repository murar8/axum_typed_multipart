use crate::TypedMultipartError;
use async_trait::async_trait;
use axum::extract::Multipart;

/// Types that can be created from multipart form data.
///
/// Structs that implement this trait can be used with the [TypedMultipart](crate::TypedMultipart)
/// extractor to parse multipart requests into strongly-typed data structures.
///
/// ## Example
///
/// ```rust
/// use axum_typed_multipart::TryFromMultipart;
///
/// #[derive(TryFromMultipart)]
/// struct Data {
///     name: String,
/// }
/// ```
#[async_trait]
pub trait TryFromMultipart: Sized {
    async fn try_from_multipart(multipart: &mut Multipart) -> Result<Self, TypedMultipartError>;
}

/// Stateful variant of [TryFromMultipart] that provides access to application state during parsing.
///
/// This enables validation or transformation based on application-specific context.
///
/// ## Example
///
/// ```rust,no_run
#[doc = include_str!("../examples/state.rs")]
/// ```
#[async_trait]
pub trait TryFromMultipartWithState<S>: Sized {
    /// Creates an instance from multipart data with access to application state.
    async fn try_from_multipart_with_state(
        multipart: &mut Multipart,
        state: &S,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<T, S> TryFromMultipartWithState<S> for T
where
    T: TryFromMultipart,
{
    async fn try_from_multipart_with_state(
        multipart: &mut Multipart,
        _state: &S,
    ) -> Result<Self, TypedMultipartError> {
        T::try_from_multipart(multipart).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::Form;

    struct Data {
        name: String,
    }

    #[async_trait]
    impl TryFromMultipart for Data {
        async fn try_from_multipart(
            multipart: &mut Multipart,
        ) -> Result<Self, TypedMultipartError> {
            let field = multipart.next_field().await.unwrap().unwrap();
            Ok(Data { name: field.text().await.unwrap() })
        }
    }

    #[tokio::test]
    async fn test_try_from_multipart_with_state() {
        let handler = |mut multipart: Multipart| async move {
            let res = Data::try_from_multipart_with_state(&mut multipart, &()).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap().name, "Hello, world!");
        };
        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().text("name", "Hello, world!"))
            .await;
    }
}
