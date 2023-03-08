mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_request_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    #[form_data(field_name = "renamed_field")]
    field: u8,
}

#[tokio::test]
async fn test_renamed_field() {
    let mut form = Form::default();
    form.add_text("renamed_field", "42");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.field, 42);
}
