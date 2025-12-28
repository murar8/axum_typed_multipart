#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

/// The fields are declared this way to make sure the derive macro supports all
/// [Option] signatures.
#[derive(TryFromMultipart)]
struct Data {
    option_field: Option<String>,
    std_option_field: std::option::Option<String>,
    core_option_field: core::option::Option<String>,
}

#[tokio::test]
async fn test_option() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.option_field, Some(String::from("John")));
        assert_eq!(data.std_option_field, None);
        assert_eq!(data.core_option_field, None);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("option_field", "John"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
