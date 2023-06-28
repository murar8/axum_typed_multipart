mod util;

use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

/// The fields are declared this way to make sure the derive macro supports
/// all [Option] signatures.
#[derive(TryFromMultipart)]
struct Foo {
    option_field_0: std::option::Option<u8>,
    option_field_1: core::option::Option<u8>,
    option_field_2: Option<u8>,
}

#[tokio::test]
async fn test_option_populated() {
    let mut form = Form::default();
    form.add_text("option_field_0", "0");
    form.add_text("option_field_1", "1");
    form.add_text("option_field_2", "2");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.option_field_0, Some(0));
    assert_eq!(data.option_field_1, Some(1));
    assert_eq!(data.option_field_2, Some(2));
}

#[tokio::test]
async fn test_option_empty() {
    let mut form = Form::default();
    form.add_text("other_field", "0");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.option_field_0, None);
    assert_eq!(data.option_field_1, None);
    assert_eq!(data.option_field_2, None);
}
