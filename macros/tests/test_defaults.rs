use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;
use reqwest::StatusCode;

#[derive(TryFromMultipart)]
struct Foo {
    #[form_data(default)]
    field: String,

    #[form_data(default)]
    optional_field: Option<String>,

    #[form_data(default)]
    list_field: Vec<String>,
}

#[tokio::test]
async fn test_defaults() {
    async fn handler(TypedMultipart(foo): TypedMultipart<Foo>) {
        assert_eq!(foo.field, "");
        assert_eq!(foo.optional_field, Option::default());
        assert_eq!(foo.list_field, Vec::<String>::default());
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("foo", "bar"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
