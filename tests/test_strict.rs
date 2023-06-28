mod util;

use axum_typed_multipart::{TryFromMultipart, TypedMultipartError};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

#[derive(TryFromMultipart, Debug)]
#[try_from_multipart(strict)]
struct Foo {
    first_name: String,
    items: Vec<String>,
}

#[tokio::test]
async fn test_strict() {
    let mut form = Form::default();
    form.add_text("first_name", "John");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.first_name, "John");
}

#[tokio::test]
async fn test_strict_list() {
    let mut form = Form::default();
    form.add_text("first_name", "John");
    form.add_text("items", "bread");
    form.add_text("items", "cheese");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.items, vec![String::from("bread"), String::from("cheese")]);
}

#[tokio::test]
async fn test_strict_duplicate_field() {
    let mut form = Form::default();
    form.add_text("first_name", "John");
    form.add_text("first_name", "Frank");

    let error = get_typed_multipart_from_form::<Foo>(form).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::DuplicateField { .. }));
}

#[tokio::test]
async fn test_strict_unknown_field() {
    let mut form = Form::default();
    form.add_text("first_name", "John");
    form.add_text("last_name", "John");

    let error = get_typed_multipart_from_form::<Foo>(form).await.unwrap_err();

    assert!(matches!(error, TypedMultipartError::UnknownField { .. }));
}
