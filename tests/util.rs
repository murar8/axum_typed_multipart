use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart, TypedMultipartError};
use common_multipart_rfc7578::client::multipart::{Body, Form};
use futures_util::TryStreamExt;

pub async fn get_request_from_form(form: Form<'_>) -> Request<String> {
    let content_type = form.content_type();

    let body = Body::from(form).try_concat().await.unwrap();
    let body = String::from_utf8(Vec::from(body)).unwrap();

    Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, content_type)
        .body(body)
        .unwrap()
}

pub async fn get_typed_multipart_from_form<T: TryFromMultipart>(
    form: Form<'_>,
) -> Result<TypedMultipart<T>, TypedMultipartError> {
    let request = get_request_from_form(form).await;
    TypedMultipart::<T>::from_request(request, &()).await
}
