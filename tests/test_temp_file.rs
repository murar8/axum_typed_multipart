mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TempFile, TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use std::fs::read_to_string;
use std::io::BufReader;
use tempfile::tempdir;
use util::get_request_from_form;

#[derive(TryFromMultipart)]
struct Foo {
    file: TempFile,
}

#[tokio::test]
async fn test_temp_file() {
    let mut form = Form::default();

    form.add_reader_file_with_mime(
        "file",
        BufReader::new("Potato!".as_bytes()),
        "potato.txt",
        mime::TEXT_PLAIN,
    );

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("potato.txt");

    data.file.persist(&file_path, false).await.unwrap();

    let data = read_to_string(&file_path).unwrap();

    assert_eq!(data, "Potato!");
}
