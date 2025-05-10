#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct Data {
    name: String,
    items: Vec<String>,
    #[form_data(default)]
    default_value: String,
}

#[tokio::test]
async fn test_strict() {
    async fn handler(TypedMultipart(data): TypedMultipart<Data>) {
        assert_eq!(data.name, "data");
        assert_eq!(data.items, vec!["bread", "cheese"]);
        assert_eq!(data.default_value, "default");
    }

    let form = Form::new()
        .text("name", "data")
        .text("items", "bread")
        .text("items", "cheese")
        .text("default_value", "default");
    let res =
        TestClient::new(Router::new().route("/", post(handler))).post("/").multipart(form).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_strict_unknown_field() {
    async fn handler(_: TypedMultipart<Data>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("unknown_field", "data"))
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'unknown_field' is not expected");
}

#[tokio::test]
async fn test_strict_duplicate_field() {
    async fn handler(_: TypedMultipart<Data>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("name", "data").text("name", "bar"))
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'name' is already present");
}

#[tokio::test]
async fn test_strict_missing_field_name() {
    async fn handler(_: TypedMultipart<Data>) {
        panic!("should not be called");
    }

    // TODO: The multipart/form-data spec allows for having fields without a
    // name, but reqwest does not support adding them to the form. Currently we
    // are only testing fields with empty name but not missing ones. We should
    // find a way to test this.
    let form = Form::new().text("", "data");

    let res =
        TestClient::new(Router::new().route("/", post(handler))).post("/").multipart(form).await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field name is empty");
}
