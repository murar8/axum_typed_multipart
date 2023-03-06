use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
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
