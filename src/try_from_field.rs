use crate::try_from_chunks::TryFromChunks;
use crate::{FieldMetadata, TypedMultipartError};
use axum::async_trait;
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

#[async_trait]
impl<T> TryFromField for T
where
    T: TryFromChunks + Send + Sync + 'static,
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
mod tests {
    use super::*;
    use axum::extract::Multipart;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use futures_core::Stream;
    use reqwest::multipart::Form;

    struct Foo(String);

    #[async_trait]
    impl TryFromChunks for Foo {
        async fn try_from_chunks(
            chunks: impl Stream<Item = Result<bytes::Bytes, TypedMultipartError>> + Send + Sync,
            metadata: FieldMetadata,
        ) -> Result<Self, TypedMultipartError> {
            let data = String::try_from_chunks(chunks, metadata).await?;
            Ok(Self(data))
        }
    }

    #[tokio::test]
    async fn test_try_from_field() {
        async fn handler(mut multipart: Multipart) {
            let field = multipart.next_field().await.unwrap().unwrap();
            let foo = Foo::try_from_field(field, None).await.unwrap();
            assert_eq!(foo.0, "Hello, world!");
        }

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().text("data", "Hello, world!"))
            .send()
            .await;
    }
}
