use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::{Body, Form};
use futures_util::TryStreamExt;

#[derive(TryFromMultipart)]
struct Bar {
    name: String,
    #[form_data(field_name = "site_url")]
    url: String,
}

#[tokio::test]
async fn test_typed_multipart() {
    let mut form = Form::default();
    form.add_text("name", "john");
    form.add_text("site_url", "https://www.rust-lang.org/");

    let content_type = form.content_type();

    let body = Body::from(form).try_concat().await.unwrap();
    let body = String::from_utf8(Vec::from(body)).unwrap();

    let request = Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, content_type)
        .body(body)
        .unwrap();

    let bar = TypedMultipart::<Bar>::from_request(request, &()).await.unwrap().0;

    assert_eq!(bar.name, "john");
    assert_eq!(bar.url, "https://www.rust-lang.org/");
}
