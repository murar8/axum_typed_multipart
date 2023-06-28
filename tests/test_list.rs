mod util;

use axum_typed_multipart::TryFromMultipart;
use common_multipart_rfc7578::client::multipart::Form;
use util::get_typed_multipart_from_form;

/// The fields are declared this way to make sure the derive macro supports
/// all [Vec] signatures.
#[derive(TryFromMultipart)]
struct Foo {
    items: Vec<String>,
    names: std::vec::Vec<String>,
}

#[tokio::test]
async fn test_list() {
    let mut form = Form::default();
    form.add_text("items", "bread");
    form.add_text("items", "cheese");

    let data = get_typed_multipart_from_form::<Foo>(form).await.unwrap().0;

    assert_eq!(data.items, vec![String::from("bread"), String::from("cheese")]);
    assert_eq!(data.names, Vec::<String>::new());
}
