mod util;

use axum_typed_multipart::{TempFile, TryFromMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use std::fs::read_to_string;
use std::io::BufReader;
use tempfile::TempDir;
use util::get_typed_multipart_from_form;

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

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    let file_path = TempDir::new().unwrap().into_path().join("potato.txt");
    data.file.persist(&file_path, false).unwrap();
    let contents = read_to_string(&file_path).unwrap();

    assert_eq!(contents, "Potato!");
}
