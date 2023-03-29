use crate::TypedMultipartError;
use axum::async_trait;
use axum::extract::Multipart;

/// Types that can be created from an instance of [Multipart].
///
/// Structs that implement this trait can be used as type parameters for
/// [TypedMultipart](crate::TypedMultipart) allowing to
/// generate the supplied struct from the request data.
///
/// ## Derive macro
///
/// The trait can be implemented automatically using the corresponding derive
/// macro.
///
/// All fields for the supplied struct must implement the
/// [TryFromField](crate::TryFromField) trait to be able to
/// derive the trait.
///
/// An error will be returned if at least one field is missing, with the
/// exception of [Option] types, which will be set as [Option::None].
///
/// ### `form_data` attribute
///
/// Can be applied to the struct fields to configure the parser behaviour.
///
/// #### Arguments
///
/// - `field_name` => Can be used to configure a different name for the source
/// field in the incoming form data.
///
/// - `default` => Populate the field using the type's [Default] implementation
/// when the field is not supplied in the request.
///
/// ## Example
///
/// ```rust
/// use axum_typed_multipart::TryFromMultipart;
///
/// #[derive(TryFromMultipart)]
/// struct Foo {
///     name: String,
/// }
/// ```
#[async_trait]
pub trait TryFromMultipart: Sized {
    async fn try_from_multipart(multipart: &mut Multipart) -> Result<Self, TypedMultipartError>;
}
