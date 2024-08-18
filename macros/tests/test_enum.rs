#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromField, TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(Debug, PartialEq, TryFromField)]
#[try_from_field(rename_all = "UPPERCASE")]
pub enum Interest {
    #[field(rename = "RUNNING")]
    Jogging,
    Music,
    Programming,
}

#[derive(TryFromMultipart)]
pub struct Data {
    pub name: String,
    pub interests: Vec<Interest>,
}

#[tokio::test]
async fn test_enum() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.name, "John");
        assert_eq!(data.interests[0], Interest::Jogging);
        assert_eq!(data.interests[1], Interest::Music);
        assert_eq!(data.interests[2], Interest::Programming);
    };

    let form = Form::new()
        .text("name", "John")
        .text("interests", "RUNNING")
        .text("interests", "MUSIC")
        .text("interests", "PROGRAMMING");

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .await
        .post("/")
        .multipart(form)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
