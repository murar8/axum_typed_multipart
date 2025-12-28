#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Data {
    bool_field: bool,
}

#[tokio::test]
async fn test_bool_true() {
    for field in
        ["true", "True", "TRUE", "true", "1", "y", "Y", "yes", "Yes", "YES", "on", "On", "ON"]
    {
        let res = TestClient::new(Router::new().route(
            "/",
            post(|TypedMultipart(data): TypedMultipart<Data>| async move {
                assert!(data.bool_field);
            }),
        ))
        .post("/")
        .multipart(Form::new().text("bool_field", field))
        .send()
        .await
        .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_bool_false() {
    for field in
        ["false", "False", "FALSE", "false", "0", "n", "N", "no", "No", "NO", "off", "Off", "OFF"]
    {
        let res = TestClient::new(Router::new().route(
            "/",
            post(|TypedMultipart(data): TypedMultipart<Data>| async move {
                assert!(!data.bool_field);
            }),
        ))
        .post("/")
        .multipart(Form::new().text("bool_field", field))
        .send()
        .await
        .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
