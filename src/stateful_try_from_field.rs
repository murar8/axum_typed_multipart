use crate::{util, FieldMetadata, StatefulTryFromChunks, TypedMultipartError};
use async_trait::async_trait;
use axum::extract::multipart::Field;

/// Types that can be created from an instance of [Field] with a state.
#[async_trait]
pub trait StatefulTryFromField<S>: Sized {
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl<T, S> StatefulTryFromField<S> for T
where
    T: StatefulTryFromChunks<S>,
    S: Sync,
{
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        state: &S,
    ) -> Result<Self, TypedMultipartError> {
        let metadata = FieldMetadata::from(&field);
        let mut field_name = metadata.name.clone().unwrap_or(String::new());
        let chunks = util::get_chunks(field, limit_bytes, &mut field_name);
        T::try_from_chunks_with_state(chunks, metadata, state).await
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use crate::FieldMetadata;
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
    impl StatefulTryFromChunks<()> for Data {
        async fn try_from_chunks_with_state(
            chunks: impl Stream<Item = Result<bytes::Bytes, TypedMultipartError>> + Send + Sync + Unpin,
            metadata: FieldMetadata,
            state: &(),
        ) -> Result<Self, TypedMultipartError> {
            let data = String::try_from_chunks_with_state(chunks, metadata, state).await?;
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
            let res = Data::try_from_field_with_state(field, Some(512), &()).await;
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
