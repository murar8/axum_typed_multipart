use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart, TypedMultipartError};
use axum_typed_multipart_core::field_data::FieldData;
use axum_typed_multipart_core::temp_file::TempFile;
use common_multipart_rfc7578::client::multipart::{Body, Form};
use futures_util::TryStreamExt;
use std::fs::read_to_string;
use std::io::BufReader;
use tempfile::tempdir;

#[derive(TryFromMultipart, Debug)]
struct Simple {
    #[allow(dead_code)]
    field: u8,
}

#[derive(TryFromMultipart)]
struct Renamed {
    #[form_data(field_name = "renamed_field")]
    field: u8,
}

#[derive(TryFromMultipart)]
struct FileUploadMemory {
    #[form_data(field_name = "input_file")]
    file: FieldData<String>,
}

#[derive(TryFromMultipart)]
struct FileUploadFS {
    file: TempFile,
}

#[derive(TryFromMultipart)]
struct PrimitiveTypes {
    i8_field: i8,
    i16_field: i16,
    i32_field: i32,
    i64_field: i64,
    i128_field: i128,
    isize_field: isize,
    u8_field: u8,
    u16_field: u16,
    u32_field: u32,
    u64_field: u64,
    u128_field: u128,
    usize_field: usize,
    f32_field: f32,
    f64_field: f64,
    bool_field: bool,
    char_field: char,
    string_field: String,
}

/// The fields are declared this way to make sure the derive macro supports
/// all [Option] signatures.
#[derive(TryFromMultipart)]
struct OptionVariants {
    option_field_0: std::option::Option<u8>,
    option_field_1: core::option::Option<u8>,
    option_field_2: Option<u8>,
}

async fn get_request_from_form(form: Form<'_>) -> Request<String> {
    let content_type = form.content_type();

    let body = Body::from(form).try_concat().await.unwrap();
    let body = String::from_utf8(Vec::from(body)).unwrap();

    Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, content_type)
        .body(body)
        .unwrap()
}

#[tokio::test]
async fn test_primitive_types() {
    let mut form = Form::default();
    form.add_text("i8_field", "-42");
    form.add_text("i16_field", "-42");
    form.add_text("i32_field", "-42");
    form.add_text("i64_field", "-42");
    form.add_text("i128_field", "-42");
    form.add_text("isize_field", "-42");
    form.add_text("u8_field", "42");
    form.add_text("u16_field", "42");
    form.add_text("u32_field", "42");
    form.add_text("u64_field", "42");
    form.add_text("u128_field", "42");
    form.add_text("usize_field", "42");
    form.add_text("f32_field", "42.5");
    form.add_text("f64_field", "42.5");
    form.add_text("bool_field", "true");
    form.add_text("char_field", "$");
    form.add_text("string_field", "Hello, world!");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<PrimitiveTypes>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.i8_field, -42);
    assert_eq!(data.i16_field, -42);
    assert_eq!(data.i32_field, -42);
    assert_eq!(data.i64_field, -42);
    assert_eq!(data.i128_field, -42);
    assert_eq!(data.isize_field, -42);
    assert_eq!(data.u8_field, 42);
    assert_eq!(data.u16_field, 42);
    assert_eq!(data.u32_field, 42);
    assert_eq!(data.u64_field, 42);
    assert_eq!(data.u128_field, 42);
    assert_eq!(data.usize_field, 42);
    assert_eq!(data.f32_field, 42.5);
    assert_eq!(data.f64_field, 42.5);
    assert!(data.bool_field);
    assert_eq!(data.char_field, '$');
    assert_eq!(data.string_field, "Hello, world!");
}

#[tokio::test]
async fn test_option_populated() {
    let mut form = Form::default();
    form.add_text("option_field_0", "0");
    form.add_text("option_field_1", "1");
    form.add_text("option_field_2", "2");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<OptionVariants>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.option_field_0, Some(0));
    assert_eq!(data.option_field_1, Some(1));
    assert_eq!(data.option_field_2, Some(2));
}

#[tokio::test]
async fn test_option_empty() {
    let mut form = Form::default();
    form.add_text("other_field", "0");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<OptionVariants>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.option_field_0, None);
    assert_eq!(data.option_field_1, None);
    assert_eq!(data.option_field_2, None);
}

#[tokio::test]
async fn test_renamed_field() {
    let mut form = Form::default();
    form.add_text("renamed_field", "42");

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Renamed>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.field, 42);
}

#[tokio::test]
async fn test_invalid_request() {
    let mut form = Form::default();
    form.add_text("field", "hello");

    let request = Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, "multipart/form-data") // Missing boundary.
        .body(String::from(""))
        .unwrap();

    let error = TypedMultipart::<Simple>::from_request(request, &()).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::InvalidRequest { .. }));
}

#[tokio::test]
async fn test_invalid_request_body() {
    let mut form = Form::default();
    form.add_text("field", "hello");

    let request = Request::builder()
        .uri("https://www.rust-lang.org/")
        .method("POST")
        .header(CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
        .body(String::from("WRONG_DATA"))
        .unwrap();

    let error = TypedMultipart::<Simple>::from_request(request, &()).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::InvalidRequestBody { .. }));
}

#[tokio::test]
async fn test_missing_field() {
    let mut form = Form::default();
    form.add_text("other_field", "42");

    let request = get_request_from_form(form).await;
    let error = TypedMultipart::<Simple>::from_request(request, &()).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::MissingField { .. }));
}

#[tokio::test]
async fn test_wrong_field_type() {
    let mut form = Form::default();
    form.add_text("field", "hello");

    let request = get_request_from_form(form).await;
    let error = TypedMultipart::<Simple>::from_request(request, &()).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::WrongFieldType { .. }));
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
    let data = TypedMultipart::<FileUploadMemory>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.file.metadata.name, Some(String::from("input_file")));
    assert_eq!(data.file.metadata.file_name, Some(String::from("potato.txt")));
    assert_eq!(data.file.metadata.content_type, Some(String::from("text/plain")));
    assert_eq!(data.file.contents, "Potato!");
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
    let data = TypedMultipart::<FileUploadFS>::from_request(request, &()).await.unwrap().0;

    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("potato.txt");

    data.file.persist(&file_path, false).await.unwrap();

    let data = read_to_string(&file_path).unwrap();

    assert_eq!(data, "Potato!");
}
