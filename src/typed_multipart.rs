use crate::{TryFromMultipart, TypedMultipartError};
use axum::body::{Bytes, HttpBody};
use axum::extract::{FromRequest, Multipart};
use axum::http::Request;
use axum::{async_trait, BoxError};

/// Used as as an argument for [axum handlers](axum::handler::Handler).
///
/// Implements [FromRequest] when the generic argument implements the
/// [TryFromMultipart] trait.
///
/// ## Example
///
/// ```rust
/// use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
///
/// #[derive(TryFromMultipart)]
/// struct Foo {
///     name: String,
///     email: Option<String>,
///     #[form_data(field_name = "website_url")]
///     url: Option<String>,
/// }
///
/// async fn handle_foo(TypedMultipart(foo): TypedMultipart<Foo>) {
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct TypedMultipart<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for TypedMultipart<T>
where
    T: TryFromMultipart,
    B: HttpBody + Send + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = TypedMultipartError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let multipart = &mut Multipart::from_request(req, state).await?;
        let data = T::try_from_multipart(multipart).await?;
        Ok(Self(data))
    }
}
