use crate::try_from_chunks::TryFromChunks;
use crate::{FieldMetadata, TypedMultipartError};
use async_trait::async_trait;
use axum::extract::multipart::Field;
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use std::mem;

/// Types that can be created from an instance of [Field].
///
/// All fields for a given struct must implement this trait to be able to derive
/// the [TryFromMultipart](crate::TryFromMultipart) trait.
///
/// Implementing this trait directly is not recommended since it requires the
/// user to manually implement the size limit logic. Instead, implement the
/// [TryFromChunks] trait and this trait will be implemented automatically.
#[async_trait]
pub trait TryFromField: Sized {
    /// Consume the input [Field] to create the supplied type.
    ///
    /// The `limit_bytes` parameter is used to limit the size of the field. If
    /// the field is larger than the limit, an error is returned.
    async fn try_from_field(
        field: Field<'_>,
        limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError>;
}

/// Stateful variant of [TryFromField].
///
/// This trait allows you to inject application state into the parser.
#[async_trait]
pub trait TryFromFieldWithState<S>: Sized {
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        state: &S,
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

#[async_trait]
impl<T> TryFromField for T
where
    T: TryFromChunks + Send + Sync,
{
    async fn try_from_field(
        field: Field<'_>,
        limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let metadata = FieldMetadata::from(&field);
        let mut field_name = metadata.name.clone().unwrap_or(String::new());
        let mut size_bytes = 0;

        let chunks = field.map_err(TypedMultipartError::from).map(|chunk| {
            if let Ok(chunk) = chunk.as_ref() {
                size_bytes += chunk.len();

                if let Some(limit_bytes) = limit_bytes {
                    if size_bytes > limit_bytes {
                        return Err(TypedMultipartError::FieldTooLarge {
                            field_name: mem::take(&mut field_name),
                            limit_bytes,
                        });
                    }
                }
            }

            chunk
        });

        T::try_from_chunks(chunks, metadata).await
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
    use futures_core::Stream;
    use reqwest::multipart::Form;
    use std::borrow::Cow;

    #[derive(Debug)]
    struct Data(String);

    #[async_trait]
    impl TryFromChunks for Data {
        async fn try_from_chunks(
            chunks: impl Stream<Item = Result<bytes::Bytes, TypedMultipartError>> + Send + Sync + Unpin,
            metadata: FieldMetadata,
        ) -> Result<Self, TypedMultipartError> {
            let data = String::try_from_chunks(chunks, metadata).await?;
            Ok(Self(data))
        }
    }

    async fn test_try_from_field<T, F>(input: T, validator: F)
    where
        T: Into<Cow<'static, str>>,
        F: FnOnce(Result<Data, TypedMultipartError>) + Clone + Send + Sync + 'static,
    {
        let handler = |mut multipart: Multipart| async move {
            let field = multipart.next_field().await.unwrap().unwrap();
            let res = Data::try_from_field(field, Some(512)).await;
            validator(res);
        };

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().text("data", input))
            .await;
    }

    #[tokio::test]
    async fn test_try_from_field_valid() {
        let validator = |res: Result<Data, TypedMultipartError>| {
            assert_eq!(res.unwrap().0, "Hello, world!");
        };
        test_try_from_field("Hello, world!", validator).await;
    }

    #[tokio::test]
    async fn test_try_from_too_large() {
        let validator = |res: Result<Data, TypedMultipartError>| {
            assert!(matches!(res, Err(TypedMultipartError::FieldTooLarge { .. })));
        };
        test_try_from_field("x".repeat(513), validator).await;
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests_with_state {
    use super::*;
    use axum::extract::Multipart;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::Form;

    #[derive(Clone)]
    struct State(String);

    struct DataWithState(String);

    #[async_trait]
    impl TryFromFieldWithState<State> for DataWithState {
        async fn try_from_field_with_state(
            field: Field<'_>,
            limit_bytes: Option<usize>,
            state: &State,
        ) -> Result<Self, TypedMultipartError> {
            let data = String::try_from_field(field, limit_bytes).await?;
            Ok(Self(format!("{}, {}", state.0, data)))
        }
    }

    #[tokio::test]
    async fn test_try_from_field_with_state() {
        let handler = |mut multipart: Multipart| async move {
            let field = multipart.next_field().await.unwrap().unwrap();
            let state = State("Hello".to_string());
            let res = DataWithState::try_from_field_with_state(field, Some(512), &state).await;
            assert_eq!(res.unwrap().0, "Hello, world!");
        };
        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().text("data", "world!"))
            .await;
    }
}
