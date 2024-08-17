use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Data {
    field: String,
}

#[tokio::test]
async fn test_field_order() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.field, "baz");
    };

    // TODO: The multipart/form-data spec allows for having fields without a
    // name, but reqwest does not support adding them to the form. Currently we
    // are only testing fields with empty name but not missing ones. We should
    // find a way to test this.
    let form = Form::new()
        .text("field", "data")
        .text("field", "bar")
        .text("field", "baz")
        .text("unknown_field", "data") // should be ignored
        .text("", "data"); // should be ignored

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .await
        .post("/")
        .multipart(form)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_field() {
    async fn handler(_: TypedMultipart<Data>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .await
        .post("/")
        .multipart(Form::new().text("unknown_field", "data"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'field' is required");
}
