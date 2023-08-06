use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;
use reqwest::StatusCode;

#[derive(TryFromMultipart)]
struct Foo {
    standard_field: String,

    r#raw_field: String,

    #[form_data(field_name = "source_field")]
    renamed_field: String,
}

#[tokio::test]
async fn test_identifiers() {
    async fn handler(TypedMultipart(foo): TypedMultipart<Foo>) {
        assert_eq!(foo.standard_field, "foo");
        assert_eq!(foo.r#raw_field, "bar");
        assert_eq!(foo.renamed_field, "baz");
    }

    let form = Form::new()
        .text("standard_field", "foo")
        .text("raw_field", "bar")
        .text("source_field", "baz");

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(form)
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
