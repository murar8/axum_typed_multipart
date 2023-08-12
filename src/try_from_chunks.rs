use crate::{FieldMetadata, TypedMultipartError};
use axum::async_trait;
use axum::body::Bytes;
use bytes::BytesMut;
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use std::any::type_name;
use tempfile::NamedTempFile;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;

/// Types that can be created from a [Stream] of [Bytes].
///
/// All fields for a given struct must implement either this trait or the
/// [TryFromField](crate::TryFromField) trait directly to be able to derive the
/// [TryFromMultipart](crate::TryFromMultipart) trait.
///
/// ## Example
///
/// ```rust
/// use axum::async_trait;
/// use axum::body::Bytes;
/// use axum_typed_multipart::{FieldMetadata, TryFromChunks, TypedMultipartError};
/// use futures_util::stream::Stream;
///
/// struct Data(String);
///
/// #[async_trait]
/// impl TryFromChunks for Data {
///     async fn try_from_chunks(
///         chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
///         metadata: FieldMetadata,
///     ) -> Result<Self, TypedMultipartError> {
///         let string = String::try_from_chunks(chunks, metadata).await?;
///         Ok(Data(string))
///     }
/// }
/// ```
#[async_trait]
pub trait TryFromChunks: Sized {
    /// Consume the input [Stream] of [Bytes] to create the supplied type.
    ///
    /// The `metadata` parameter contains information about the field.
    async fn try_from_chunks(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
    ) -> Result<Self, TypedMultipartError>;
}

#[async_trait]
impl TryFromChunks for Bytes {
    async fn try_from_chunks(
        mut chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        _: FieldMetadata,
    ) -> Result<Self, TypedMultipartError> {
        let mut bytes = BytesMut::new();

        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            bytes.extend_from_slice(&chunk);
        }

        Ok(bytes.freeze())
    }
}

#[async_trait]
impl TryFromChunks for String {
    async fn try_from_chunks(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
    ) -> Result<Self, TypedMultipartError> {
        let field_name = metadata.name.clone().unwrap().to_string();
        let bytes = Bytes::try_from_chunks(chunks, metadata).await?;

        String::from_utf8(bytes.into()).map_err(|_| TypedMultipartError::WrongFieldType {
            field_name,
            wanted_type: type_name::<String>().to_string(),
        })
    }
}

/// Generate a [TryFromChunks] implementation for the supplied data type using
/// the `str::parse` method on the textual representation of the field data.
macro_rules! gen_try_from_chunks_impl {
    ( $type: ty ) => {
        #[async_trait]
        impl TryFromChunks for $type {
            async fn try_from_chunks(
                chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync+ Unpin,
                metadata: FieldMetadata,
            ) -> Result<Self, TypedMultipartError> {
                let field_name = metadata.name.clone().unwrap().to_string();
                let text = String::try_from_chunks(chunks, metadata).await?;

                str::parse(&text).map_err(|_| TypedMultipartError::WrongFieldType {
                    field_name,
                    wanted_type: type_name::<$type>().to_string(),
                })
            }
        }
    };
}

gen_try_from_chunks_impl!(i8);
gen_try_from_chunks_impl!(i16);
gen_try_from_chunks_impl!(i32);
gen_try_from_chunks_impl!(i64);
gen_try_from_chunks_impl!(i128);
gen_try_from_chunks_impl!(isize);
gen_try_from_chunks_impl!(u8);
gen_try_from_chunks_impl!(u16);
gen_try_from_chunks_impl!(u32);
gen_try_from_chunks_impl!(u64);
gen_try_from_chunks_impl!(u128);
gen_try_from_chunks_impl!(usize);
gen_try_from_chunks_impl!(f32);
gen_try_from_chunks_impl!(f64);
gen_try_from_chunks_impl!(bool); // TODO?: Consider accepting any thruthy value.
gen_try_from_chunks_impl!(char);

#[async_trait]
impl TryFromChunks for NamedTempFile {
    async fn try_from_chunks(
        mut chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        _: FieldMetadata,
    ) -> Result<Self, TypedMultipartError> {
        let temp_file = NamedTempFile::new().map_err(anyhow::Error::new)?;
        let std_file = temp_file.reopen().map_err(anyhow::Error::new)?;
        let mut async_file = AsyncFile::from_std(std_file);

        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            async_file.write_all(&chunk).await.map_err(anyhow::Error::new)?;
        }

        async_file.flush().await.map_err(anyhow::Error::new)?;

        Ok(temp_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures_util::stream;
    use std::fmt::Debug;
    use std::io::Read;

    fn create_chunks(
        value: impl Into<Bytes>,
    ) -> impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin {
        let mut chunks = Vec::<Result<Bytes, TypedMultipartError>>::new();

        for chunk in value.into().chunks(3) {
            chunks.push(Ok(Bytes::from(Vec::from(chunk))));
        }

        stream::iter(chunks)
    }

    async fn test_try_from_chunks_valid<T>(input: impl Into<Bytes>, expected: impl Into<T>)
    where
        T: TryFromChunks + PartialEq + Debug + Send + Sync + Unpin,
    {
        let chunks = create_chunks(input);
        let metadata = FieldMetadata { name: Some("test".into()), ..Default::default() };
        let res = T::try_from_chunks(chunks, metadata).await.unwrap();
        assert_eq!(res, expected.into());
    }

    async fn test_try_from_chunks_invalid<T>(input: impl Into<Bytes>)
    where
        T: TryFromChunks + PartialEq + Debug + Send + Sync + Unpin,
    {
        let chunks = create_chunks(input);
        let metadata = FieldMetadata { name: Some("test".into()), ..Default::default() };
        let res = T::try_from_chunks(chunks, metadata).await;
        assert!(matches!(res, Err(TypedMultipartError::WrongFieldType { .. })));
    }

    #[tokio::test]
    async fn test_try_from_chunks_bytes() {
        test_try_from_chunks_valid::<Bytes>("asd", "asd").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_string() {
        test_try_from_chunks_valid::<String>("asd", "asd").await;
        test_try_from_chunks_invalid::<String>(Bytes::from(vec![0x80])).await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_i8() {
        test_try_from_chunks_valid::<i8>("-42", -42).await;
        test_try_from_chunks_invalid::<i8>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_i16() {
        test_try_from_chunks_valid::<i16>("-42", -42i16).await;
        test_try_from_chunks_invalid::<i16>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_i32() {
        test_try_from_chunks_valid::<i32>("-42", -42).await;
        test_try_from_chunks_invalid::<i32>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_i64() {
        test_try_from_chunks_valid::<i64>("-42", -42).await;
        test_try_from_chunks_invalid::<i64>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_i128() {
        test_try_from_chunks_valid::<i128>("-42", -42).await;
        test_try_from_chunks_invalid::<i128>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_isize() {
        test_try_from_chunks_valid::<isize>("-42", -42isize).await;
        test_try_from_chunks_invalid::<isize>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_u8() {
        test_try_from_chunks_valid::<u8>("42", 42).await;
        test_try_from_chunks_invalid::<u8>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_u16() {
        test_try_from_chunks_valid::<u16>("42", 42u16).await;
        test_try_from_chunks_invalid::<u16>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_u32() {
        test_try_from_chunks_valid::<u32>("42", 42u32).await;
        test_try_from_chunks_invalid::<u32>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_u64() {
        test_try_from_chunks_valid::<u64>("42", 42u64).await;
        test_try_from_chunks_invalid::<u64>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_u128() {
        test_try_from_chunks_valid::<u128>("42", 42u128).await;
        test_try_from_chunks_invalid::<u128>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_usize() {
        test_try_from_chunks_valid::<usize>("42", 42usize).await;
        test_try_from_chunks_invalid::<usize>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_f32() {
        test_try_from_chunks_valid::<f32>("42.0", 42.0).await;
        test_try_from_chunks_invalid::<f32>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_f64() {
        test_try_from_chunks_valid::<f64>("42.0", 42.0).await;
        test_try_from_chunks_invalid::<f64>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_bool() {
        test_try_from_chunks_valid::<bool>("true", true).await;
        test_try_from_chunks_valid::<bool>("false", false).await;
        test_try_from_chunks_invalid::<bool>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_char() {
        test_try_from_chunks_valid::<char>("a", 'a').await;
        test_try_from_chunks_invalid::<char>("invalid").await;
    }

    #[tokio::test]
    async fn test_try_from_chunks_named_temp_file() {
        let chunks = create_chunks("Hello, dear world!");
        let metadata = FieldMetadata { name: Some("test".into()), ..Default::default() };
        let file = NamedTempFile::try_from_chunks(chunks, metadata).await.unwrap();

        let mut buffer = String::new();
        file.reopen().unwrap().read_to_string(&mut buffer).unwrap();

        assert_eq!(buffer, "Hello, dear world!");
    }
}
