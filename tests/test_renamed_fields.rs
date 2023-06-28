mod util;

use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    #[form_data(field_name = "renamed_field")]
    field: u8,
}

#[tokio::test]
async fn test_renamed_field() {
    let mut form = Form::default();
    form.add_text("renamed_field", "42");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.field, 42);
}
