mod util;

use axum::body::Bytes;
use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart)]
struct Foo {
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
    bytes_field: Bytes,
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
    form.add_text("bytes_field", "123");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

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
    assert_eq!(data.bytes_field, "123");
}
