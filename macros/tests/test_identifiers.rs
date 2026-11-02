use axum::Router;
use axum::http::StatusCode;
use axum::routing::post;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Data {
    standard: String,

    r#raw: String,

    #[form_data(field_name = "source")]
    renamed: String,
}

#[tokio::test]
async fn identifiers() {
    let handler = |TypedMultipart(data): TypedMultipart<Data>| async move {
        assert_eq!(data.standard, "data");
        assert_eq!(data.r#raw, "bar");
        assert_eq!(data.renamed, "baz");
    };

    let form = Form::new().text("standard", "data").text("raw", "bar").text("source", "baz");

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[derive(TryFromMultipart)]
#[try_from_multipart(rename_all = "camelCase")]
struct CamelCaseData {
    my_field: String,
}

#[tokio::test]
async fn rename_all() {
    let handler = |TypedMultipart(data): TypedMultipart<CamelCaseData>| async move {
        assert_eq!(data.my_field, "value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("myField", "value"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
