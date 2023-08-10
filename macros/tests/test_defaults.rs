use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;
use reqwest::StatusCode;

#[derive(TryFromMultipart)]
struct Data {
    #[form_data(default)]
    field: String,

    #[form_data(default)]
    optional_field: Option<String>,

    #[form_data(default)]
    list_field: Vec<String>,
}

#[tokio::test]
async fn test_defaults() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.field, "");
        assert_eq!(data.optional_field, Option::default());
        assert_eq!(data.list_field, Vec::<String>::default());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("data", "bar"))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
