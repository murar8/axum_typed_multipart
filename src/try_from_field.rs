use crate::TypedMultipartError;
use axum::async_trait;
use axum::body::Bytes;
use axum::extract::multipart::Field;
use bytes::BytesMut;
use futures_core::stream::Stream;
use std::any::type_name;

/// Types that can be created from an instance of [Field].
///
/// All fields for a given struct must implement this trait to be able to derive
/// the [TryFromMultipart](crate::TryFromMultipart) trait.
///
/// ## Example
///
/// ```rust
/// use axum::async_trait;
/// use axum::body::Bytes;
/// use axum::extract::multipart::Field;
/// use axum_typed_multipart::{TryFromField, TypedMultipartError};
/// use bytes::BytesMut;
/// use futures_core::stream::Stream;
/// use std::any::type_name;
///
/// struct Foo(String);
///
/// #[async_trait]
/// impl TryFromField for Foo {
///     async fn try_from_field(
///         field: Field<'_>,
///         limit_bytes: Option<usize>,
///     ) -> Result<Self, TypedMultipartError> {
///         let text = <String as TryFromField>::try_from_field(field, limit_bytes).await?;
///         Ok(Self(text))
///     }
/// }
/// ```
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
impl TryFromField for Bytes {
    async fn try_from_field(
        mut field: Field<'_>,
        limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let initial_size = field.size_hint().1.unwrap_or(0);
        let mut data = BytesMut::with_capacity(initial_size);

        while let Some(chunk) = field.chunk().await? {
            if let Some(limit_bytes) = limit_bytes {
                if data.len() + chunk.len() > limit_bytes {
                    let field_name = field.name().unwrap().to_string();
                    return Err(TypedMultipartError::FieldTooLarge { field_name, limit_bytes });
                }
            }

            data.extend_from_slice(&chunk)
        }

        Ok(data.freeze())
    }
}

#[async_trait]
impl TryFromField for String {
    async fn try_from_field(
        field: Field<'_>,
        limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let field_name = field.name().unwrap().to_string();
        let bytes = <Bytes as TryFromField>::try_from_field(field, limit_bytes).await?;

        String::from_utf8(bytes.into()).map_err(|_| TypedMultipartError::WrongFieldType {
            field_name,
            wanted_type: type_name::<u128>().to_string(),
        })
    }
}

/// Generate a [TryFromField] implementation for the supplied data type using
/// the `str::parse` method on the text representation of the field data.
macro_rules! gen_try_from_field_impl {
    ( $type: ty ) => {
        #[async_trait]
        impl TryFromField for $type {
            async fn try_from_field(
                field: Field<'_>,
                limit_bytes: Option<usize>,
            ) -> Result<Self, TypedMultipartError> {
                let field_name = field.name().unwrap().to_string();
                let text = <String as TryFromField>::try_from_field(field, limit_bytes).await?;

                str::parse(&text).map_err(|_| TypedMultipartError::WrongFieldType {
                    field_name,
                    wanted_type: type_name::<$type>().to_string(),
                })
            }
        }
    };
}

gen_try_from_field_impl!(i8);
gen_try_from_field_impl!(i16);
gen_try_from_field_impl!(i32);
gen_try_from_field_impl!(i64);
gen_try_from_field_impl!(i128);
gen_try_from_field_impl!(isize);
gen_try_from_field_impl!(u8);
gen_try_from_field_impl!(u16);
gen_try_from_field_impl!(u32);
gen_try_from_field_impl!(u64);
gen_try_from_field_impl!(u128);
gen_try_from_field_impl!(usize);
gen_try_from_field_impl!(f32);
gen_try_from_field_impl!(f64);
gen_try_from_field_impl!(bool); // TODO?: Consider accepting any thruthy value.
gen_try_from_field_impl!(char);

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Multipart;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::{Form, Part};
    use std::borrow::Cow;
    use std::fmt::Debug;

    async fn test_try_from_field<T, U>(wanted_value: T, valid_input: U, invalid_input: Option<U>)
    where
        T: TryFromField + Debug + PartialEq + Clone + Send + Sync + 'static,
        U: Into<Cow<'static, str>>,
    {
        let handler = |mut multipart: Multipart| async move {
            let field = multipart.next_field().await?.unwrap();
            let data = <T as TryFromField>::try_from_field(field, Some(512)).await?;
            assert_eq!(data, wanted_value);
            Ok::<(), TypedMultipartError>(())
        };

        let client = TestClient::new(Router::new().route("/", post(handler)));

        let form = Form::new().part("data", Part::text(valid_input));
        let res = client.post("/").multipart(form).send().await;

        assert_eq!(res.status(), StatusCode::OK);

        if let Some(invalid_input) = invalid_input {
            let form = Form::new().part("data", Part::text(invalid_input));
            let res = client.post("/").multipart(form).send().await;
            let status = res.status();
            let msg = res.text().await;

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(msg.contains("field 'data' must be of type"));
        }

        let data = "x".repeat(513);
        let form = Form::new().part("data", Part::text(data));
        let res = client.post("/").multipart(form).send().await;
        let status = res.status();
        let msg = res.text().await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(msg, "field 'data' is larger than 512 bytes");
    }

    #[tokio::test]
    async fn test_try_from_field_bytes() {
        test_try_from_field(Bytes::from("test"), "test", None).await;
    }

    #[tokio::test]
    async fn test_try_from_field_string() {
        test_try_from_field(String::from("test"), "test", None).await;
    }

    #[tokio::test]
    async fn test_try_from_field_i8() {
        test_try_from_field(-1i8, "-1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_i16() {
        test_try_from_field(-1i16, "-1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_i32() {
        test_try_from_field(-1i32, "-1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_i64() {
        test_try_from_field(-1i64, "-1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_i128() {
        test_try_from_field(-1i128, "-1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_isize() {
        test_try_from_field(-1isize, "-1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_u8() {
        test_try_from_field(1u8, "1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_u16() {
        test_try_from_field(1u16, "1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_u32() {
        test_try_from_field(1u32, "1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_u64() {
        test_try_from_field(1u64, "1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_u128() {
        test_try_from_field(1u128, "1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_usize() {
        test_try_from_field(1usize, "1", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_f32() {
        test_try_from_field(1.5f32, "1.5", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_f64() {
        test_try_from_field(1.5f64, "1.5", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_bool() {
        test_try_from_field(true, "true", Some("x")).await;
    }

    #[tokio::test]
    async fn test_try_from_field_char() {
        test_try_from_field('a', "a", Some("abc")).await;
    }
}
