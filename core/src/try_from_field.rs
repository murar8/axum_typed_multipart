use std::any::type_name;

use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::multipart::Field;

/// Types that can be created from an instance of [Field].
///
/// All fields for a given struct must implement this trait to be able to derive
/// the [TryFromMultipart] trait.
#[async_trait]
pub trait TryFromField<'a>: Sized {
    /// Consume the input [Field] to create the supplied type.
    async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError>;
}

/// Generate a [TryFromField] implementation for the supplied type using the
/// `str::parse` method on the text representation of the field data.
macro_rules! gen_try_from_field_impl {
    ( $type: ty ) => {
        #[async_trait]
        impl<'a> TryFromField<'a> for $type {
            async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError> {
                let field_name = field.name().unwrap().to_string();
                let text = field.text().await?;

                str::parse(&text).map_err(move |_| TypedMultipartError::InvalidFieldType {
                    field_name,
                    field_type: type_name::<$type>().to_string(),
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
gen_try_from_field_impl!(bool);
gen_try_from_field_impl!(char);

#[async_trait]
impl<'a> TryFromField<'a> for String {
    async fn try_from_field(field: Field<'a>) -> Result<Self, TypedMultipartError> {
        let text = field.text().await?;
        Ok(text)
    }
}
