use axum::Router;
use axum::http::StatusCode;
use axum::routing::post;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

/// The fields are declared this way to make sure the derive macro supports all
/// [`Option`] signatures.
#[derive(TryFromMultipart)]
struct Data {
    option: Option<String>,
    std_option: std::option::Option<String>,
    core_option: core::option::Option<String>,
}

#[tokio::test]
async fn test_option() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.option, Some(String::from("John")));
        assert_eq!(data.std_option, None);
        assert_eq!(data.core_option, None);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("option", "John"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
