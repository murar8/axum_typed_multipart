mod util;

use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart, TypedMultipartError};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart, Debug)]
struct Foo {
    #[allow(dead_code)]
    field: u8,
}

#[tokio::test]
async fn test_invalid_request() {
    let mut form = Form::default();
    form.add_text("field", "hello");

    let request = Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, "multipart/form-data") // Missing boundary.
        .body(String::from(""))
        .unwrap();

    let error = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::InvalidRequest { .. }));
}

#[tokio::test]
async fn test_invalid_request_body() {
    let mut form = Form::default();
    form.add_text("field", "hello");

    let request = Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
        .body(String::from("WRONG_DATA"))
        .unwrap();

    let error = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::InvalidRequestBody { .. }));
}

#[tokio::test]
async fn test_missing_field() {
    let mut form = Form::default();
    form.add_text("other_field", "42");

    let error = get_typed_multipart_from_form::<Foo>(form).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::MissingField { .. }));
}

#[tokio::test]
async fn test_wrong_field_type() {
    let mut form = Form::default();
    form.add_text("field", "hello");

    let error = get_typed_multipart_from_form::<Foo>(form).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::WrongFieldType { .. }));
}
