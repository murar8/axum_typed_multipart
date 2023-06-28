mod util;

use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    #[form_data(default)]
    first_name: String,
    #[form_data(default)]
    last_name: String,
}

#[tokio::test]
async fn test_default() {
    let mut form = Form::default();
    form.add_text("first_name", "John");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.first_name, (String::from("John")));
    assert_eq!(data.last_name, (String::from("")));
}
