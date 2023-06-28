mod util;

use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

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

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.string_field, "Hello, rust!");
}
