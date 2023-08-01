mod util;

use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TempFile, TryFromMultipart, TypedMultipartError};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart, Debug)]
#[allow(dead_code)]
struct Foo {
    #[form_data(limit = "2b")]
    i8_field: i8,
    #[form_data(limit = "4b")]
    i16_field: i16,
    #[form_data(limit = "8b")]
    i32_field: i32,
    #[form_data(limit = "8b")]
    i64_field: i64,
    #[form_data(limit = "8b")]
    i128_field: i128,
    #[form_data(limit = "8b")]
    isize_field: isize,
    #[form_data(limit = "2b")]
    u8_field: u8,
    #[form_data(limit = "4b")]
    u16_field: u16,
    #[form_data(limit = "8b")]
    u32_field: u32,
    #[form_data(limit = "8b")]
    u64_field: u64,
    #[form_data(limit = "8b")]
    u128_field: u128,
    #[form_data(limit = "8b")]
    usize_field: usize,
    #[form_data(limit = "8b")]
    f32_field: f32,
    #[form_data(limit = "8b")]
    f64_field: f64,
    #[form_data(limit = "4b")]
    bool_field: bool,
    #[form_data(limit = "1b")]
    char_field: char,
    #[form_data(limit = "8b")]
    string_field: String,
    #[form_data(limit = "8b")]
    bytes_field: Bytes,
    #[form_data(limit = "16")]
    file_field: FieldData<TempFile>,
}

// The values are chosen so that they are exactly the right size to fit in the
// limits.
const FOO_FIELDS: [(&str, &str); 19] = [
    ("i8_field", "42"),
    ("i16_field", "4242"),
    ("i32_field", "42424242"),
    ("i64_field", "42424242"),
    ("i128_field", "42424242"),
    ("isize_field", "42424242"),
    ("u8_field", "42"),
    ("u16_field", "4242"),
    ("u32_field", "42424242"),
    ("u64_field", "42424242"),
    ("u128_field", "42424242"),
    ("usize_field", "42424242"),
    ("f32_field", "42424242"),
    ("f64_field", "42424242"),
    ("bool_field", "true"),
    ("char_field", "$"),
    ("string_field", "eightbs!"),
    ("bytes_field", "42424242"),
    ("file_field", "4242424242424242"),
];

#[tokio::test]
async fn test_limit_below() {
    let mut form = Form::default();

    for (field, value) in FOO_FIELDS.iter() {
        form.add_text(field, *value);
    }

    let result = get_typed_multipart_from_form::<Foo>(form).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_limit_above() {
    for (field, value) in FOO_FIELDS.iter() {
        let mut form = Form::default();

        for (other_field, other_value) in FOO_FIELDS.iter() {
            if other_field != field {
                form.add_text(other_field, *other_value);
            }
        }

        // Add one more byte to the value to make it too large.
        form.add_text(field, format!("{}4", value));

        let error = get_typed_multipart_from_form::<Foo>(form).await.unwrap_err();

        assert!(matches!(error, TypedMultipartError::FieldTooLarge { .. }));
    }
}
