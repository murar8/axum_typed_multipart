use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;
use reqwest::StatusCode;

/// The fields are declared this way to make sure the derive macro supports all
/// [Vec] signatures.
#[derive(TryFromMultipart)]
struct Foo {
    vec_field: Vec<String>,
    std_vec_field: std::vec::Vec<String>,
}

#[tokio::test]
async fn test_list() {
    let handler = |TypedMultipart(foo): TypedMultipart<Foo>| async move {
        assert_eq!(foo.vec_field, vec!["Apple", "Orange"]);
        assert_eq!(foo.std_vec_field, Vec::<String>::new());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("vec_field", "Apple").text("vec_field", "Orange"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
