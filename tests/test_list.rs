mod util;

use axum::extract::FromRequest;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use common_multipart_rfc7578::client::multipart::Form;
use util::get_request_from_form;

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

    let request = get_request_from_form(form).await;
    let data = TypedMultipart::<Foo>::from_request(request, &()).await.unwrap().0;

    assert_eq!(data.items, vec![String::from("bread"), String::from("cheese")]);
    assert_eq!(data.names, Vec::<String>::new());
}
