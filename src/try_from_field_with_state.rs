use crate::{TryFromField, TypedMultipartError};
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// Types that can be created from an instance of [Field] and a state.
///
/// This is a variant of [TryFromField] that allows access to the application
/// state. If you don't need access to the state, consider using that trait
/// instead.
#[async_trait]
pub trait TryFromFieldWithState<S>: Sized {
    /// Consume the input [Field] to create the supplied type.
    ///
    /// The `limit_bytes` parameter is used to limit the size of the field. If
    /// the field is larger than the limit, an error is returned.
    ///
    /// The `state` parameter contains application state passed from the
    /// [FromRequest](axum::extract::FromRequest) implementation.
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        _state: &S,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<T, S> TryFromFieldWithState<S> for T
where
    T: TryFromField,
{
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        _state: &S,
    ) -> Result<Self, TypedMultipartError> {
        T::try_from_field(field, limit_bytes).await
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use axum::extract::Multipart;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::Form;

    struct Data(String);

    #[async_trait]
    impl TryFromField for Data {
        async fn try_from_field(
            field: Field<'_>,
            limit_bytes: Option<usize>,
        ) -> Result<Self, TypedMultipartError> {
            let string = String::try_from_field(field, limit_bytes).await?;
            Ok(Data(string))
        }
    }

    #[tokio::test]
    async fn test_try_from_field_with_state() {
        let handler = |mut multipart: Multipart| async move {
            let field = multipart.next_field().await.unwrap().unwrap();
            let res = Data::try_from_field_with_state(field, Some(512), &()).await;
            assert!(matches!(res, Ok(Data(string)) if string == "Hello, world!"));
        };

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().text("data", "Hello, world!"))
            .await;
    }

    #[tokio::test]
    async fn test_try_from_field_impl() {
        let handler = |mut multipart: Multipart| async move {
            let field = multipart.next_field().await.unwrap().unwrap();
            let res = Data::try_from_field(field, Some(512)).await;
            assert!(matches!(res, Ok(Data(string)) if string == "Hello, world!"));
        };

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().text("data", "Hello, world!"))
            .await;
    }
}
