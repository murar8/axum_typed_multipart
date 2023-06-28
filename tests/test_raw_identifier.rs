mod util;

use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    r#type: String,
}

#[tokio::test]
async fn test_raw_identifier() {
    let mut form = Form::default();
    form.add_text("type", "A");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.r#type, "A");
}
