#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Address {
    street: String,
    city: String,
}

#[derive(TryFromMultipart)]
struct Person {
    name: String,
    #[form_data(nested)]
    address: Address,
}

async fn client() -> TestClient {
    let handler = |TypedMultipart(data): TypedMultipart<Person>| async move {
        assert_eq!(data.name, "Alice");
        assert_eq!(data.address.street, "123 Main St");
        assert_eq!(data.address.city, "Springfield");
    };
    TestClient::new(Router::new().route("/", post(handler)))
}

#[tokio::test]
async fn test_nested_dot_notation() {
    let form = Form::new()
        .text("name", "Alice")
        .text("address.street", "123 Main St")
        .text("address.city", "Springfield");

    let res = client().await.post("/").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nested_bracket_notation() {
    let form = Form::new()
        .text("name", "Alice")
        .text("address[street]", "123 Main St")
        .text("address[city]", "Springfield");

    let res = client().await.post("/").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nested_missing_field() {
    async fn handler(_: TypedMultipart<Person>) {
        panic!("should not be called");
    }

    let form = Form::new().text("name", "Alice").text("address.street", "123 Main St");

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'city' is required");
}
