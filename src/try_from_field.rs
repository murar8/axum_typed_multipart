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
