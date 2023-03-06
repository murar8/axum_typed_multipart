mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_request_from_form;

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

#[derive(TryFromMultipart)]
struct RenamedFields {
    #[form_data(field_name = "renamed_field")]
    field: u8,
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
    let data = TypedMultipart::<RenamedFields>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.field, 42);
}
