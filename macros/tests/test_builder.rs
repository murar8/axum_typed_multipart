#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{MultipartBuilder, TryFromMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Data {
    name: String,
    age: u32,
}

#[tokio::test]
async fn test_builder_api() {
    let handler = |mut multipart: Multipart| async move {
        let mut builder = DataBuilder::default();

        while let Some(field) = multipart.next_field().await.unwrap() {
            let field_name = field.name().unwrap_or("").to_string();
            let result = <DataBuilder as MultipartBuilder<()>>::process_field(
                &mut builder,
                &field_name,
                field,
                &(),
            )
            .await
            .unwrap();
            assert!(result.is_none()); // None means field was processed
        }

        let data = <DataBuilder as MultipartBuilder<()>>::build(builder).unwrap();
        assert_eq!(data.name, "Alice");
        assert_eq!(data.age, 30);
    };

    let form = Form::new().text("name", "Alice").text("age", "30");

    let res =
        TestClient::new(Router::new().route("/", post(handler))).post("/").multipart(form).await;

    assert_eq!(res.status(), StatusCode::OK);
}
