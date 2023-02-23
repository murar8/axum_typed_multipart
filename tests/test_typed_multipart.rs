use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::{Body, Form};
use futures_util::TryStreamExt;

async fn get_request_from_form<'a>(form: Form<'a>) -> Request<String> {
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
    }

    let request = get_request_from_form(form).await;
    let foo = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(foo.i8_field, -42);
    assert_eq!(foo.i16_field, -42);
    assert_eq!(foo.i32_field, -42);
    assert_eq!(foo.i64_field, -42);
    assert_eq!(foo.i128_field, -42);
    assert_eq!(foo.isize_field, -42);
    assert_eq!(foo.u8_field, 42);
    assert_eq!(foo.u16_field, 42);
    assert_eq!(foo.u32_field, 42);
    assert_eq!(foo.u64_field, 42);
    assert_eq!(foo.u128_field, 42);
    assert_eq!(foo.usize_field, 42);
    assert_eq!(foo.f32_field, 42.5);
    assert_eq!(foo.f64_field, 42.5);
    assert_eq!(foo.bool_field, true);
    assert_eq!(foo.char_field, '$');
    assert_eq!(foo.string_field, "Hello, world!");
}

#[tokio::test]
async fn test_option_populated() {
    let mut form = Form::default();
    form.add_text("option_field_0", "0");
    form.add_text("option_field_1", "1");
    form.add_text("option_field_2", "2");

    #[derive(TryFromMultipart)]
    struct Foo {
        option_field_0: std::option::Option<u8>,
        option_field_1: core::option::Option<u8>,
        option_field_2: Option<u8>,
    }

    let request = get_request_from_form(form).await;
    let foo = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(foo.option_field_0, Some(0));
    assert_eq!(foo.option_field_1, Some(1));
    assert_eq!(foo.option_field_2, Some(2));
}

#[tokio::test]
async fn test_option_empty() {
    let mut form = Form::default();
    form.add_text("other_field", "0");

    #[derive(TryFromMultipart)]
    struct Foo {
        option_field: Option<u8>,
    }

    let request = get_request_from_form(form).await;
    let foo = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(foo.option_field, None);
}

#[tokio::test]
async fn test_renamed_field() {
    let mut form = Form::default();
    form.add_text("renamed_field", "42");

    #[derive(TryFromMultipart)]
    struct Foo {
        #[form_data(field_name = "renamed_field")]
        field: u8,
    }

    let request = get_request_from_form(form).await;
    let foo = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(foo.field, 42);
}
