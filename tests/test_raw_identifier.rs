mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_request_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    r#type: String,
}

#[tokio::test]
async fn test_raw_identifier() {
    let mut form = Form::default();
    form.add_text("type", "A");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.r#type, "A");
}
