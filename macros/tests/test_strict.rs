use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;
use reqwest::StatusCode;

#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct Data {
    name: String,
    items: Vec<String>,
}

#[tokio::test]
async fn test_strict() {
    async fn handler(TypedMultipart(data): TypedMultipart<Data>) {
        assert_eq!(data.name, "data");
        assert_eq!(data.items, vec!["bread", "cheese"]);
    }

    let form = Form::new().text("name", "data").text("items", "bread").text("items", "cheese");
    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(form)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_strict_unknown_field() {
    let handler = |_: TypedMultipart<Data>| async move { panic!("should not be called") };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("unknown_field", "data"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'unknown_field' is not expected");
}

#[tokio::test]
async fn test_strict_deplicate_field() {
    let handler = |_: TypedMultipart<Data>| async move { panic!("should not be called") };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("name", "data").text("name", "bar"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'name' is already present");
}
