mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_request_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    #[form_data(default)]
    first_name: String,
    #[form_data(default)]
    last_name: String,
}

#[tokio::test]
async fn test_field_data() {
    let mut form = Form::default();
    form.add_text("first_name", "John");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.first_name, (String::from("John")));
    assert_eq!(data.last_name, (String::from("")));
}
