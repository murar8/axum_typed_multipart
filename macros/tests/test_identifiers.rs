#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Data {
    standard_field: String,

    r#raw_field: String,

    #[form_data(field_name = "source_field")]
    renamed_field: String,
}

#[tokio::test]
async fn test_identifiers() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.standard_field, "data");
        assert_eq!(data.r#raw_field, "bar");
        assert_eq!(data.renamed_field, "baz");
    };

    let form = Form::new()
        .text("standard_field", "data")
        .text("raw_field", "bar")
        .text("source_field", "baz");

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[derive(TryFromMultipart)]
#[try_from_multipart(rename_all = "camelCase")]
struct CamelCaseData {
    my_field: String,
}

#[tokio::test]
async fn test_rename_all() {
    let handler = |TypedMultipart(data): TypedMultipart<CamelCaseData>| async move {
        assert_eq!(data.my_field, "value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("myField", "value"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
