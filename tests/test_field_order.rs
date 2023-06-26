mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_request_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    string_field: String,
}

#[tokio::test]
async fn test_field_order() {
    let mut form = Form::default();
    form.add_text("string_field", "Hello, world!");
    form.add_text("string_field", "Hello, cargo!");
    form.add_text("string_field", "Hello, rust!");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.string_field, "Hello, rust!");
}
