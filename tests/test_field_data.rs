mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use std::io::BufReader;
use util::get_request_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    #[form_data(field_name = "input_file")]
    file: FieldData<String>,
}

#[tokio::test]
async fn test_field_data() {
    let mut form = Form::default();

    form.add_reader_file_with_mime(
        "input_file",
        BufReader::new("Potato!".as_bytes()),
        "potato.txt",
        mime::TEXT_PLAIN,
    );

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.file.metadata.name, Some(String::from("input_file")));
    assert_eq!(data.file.metadata.file_name, Some(String::from("potato.txt")));
    assert_eq!(data.file.metadata.content_type, Some(String::from("text/plain")));
    assert_eq!(data.file.contents, "Potato!");
}
