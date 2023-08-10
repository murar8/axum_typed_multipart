use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;
use reqwest::StatusCode;

#[derive(TryFromMultipart)]
struct Data {
    field: String,
}

#[tokio::test]
async fn test_field_order() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.field, "baz");
    };

    let form = Form::new()
        .text("field", "data")
        .text("field", "bar")
        .text("field", "baz")
        .text("unknown_field", "data");

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(form)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_field() {
    let handler = |_: TypedMultipart<Data>| async { panic!("should not be called") };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("unknown_field", "data"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'field' is required");
}
