#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Phone {
    number: String,
}

#[derive(TryFromMultipart)]
struct FormWithNestedSingle {
    title: String,
    #[form_data(nested)]
    owner: Person,
}

#[derive(TryFromMultipart)]
struct FormWithNestedVec {
    title: String,
    #[form_data(nested)]
    users: Vec<Person>,
}

#[derive(TryFromMultipart)]
struct FormWithDeepNesting {
    #[form_data(nested)]
    users: Vec<UserWithPhones>,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct UserWithPhones {
    name: String,
    #[form_data(nested)]
    phones: Vec<Phone>,
}

#[tokio::test]
async fn test_nested_single() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedSingle>| async move {
        assert_eq!(data.title, "Test Form");
        assert_eq!(data.owner.name, "Alice");
        assert_eq!(data.owner.age, 30);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test Form")
                .text("owner.name", "Alice")
                .text("owner.age", "30"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nested_vec() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "User List");
        assert_eq!(data.users.len(), 2);
        assert_eq!(data.users[0].name, "Alice");
        assert_eq!(data.users[0].age, 30);
        assert_eq!(data.users[1].name, "Bob");
        assert_eq!(data.users[1].age, 25);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "User List")
                .text("users[0].name", "Alice")
                .text("users[0].age", "30")
                .text("users[1].name", "Bob")
                .text("users[1].age", "25"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deep_nesting() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithDeepNesting>| async move {
        assert_eq!(data.users.len(), 2);
        assert_eq!(data.users[0].name, "Alice");
        assert_eq!(data.users[0].phones.len(), 2);
        assert_eq!(data.users[0].phones[0].number, "111");
        assert_eq!(data.users[0].phones[1].number, "222");
        assert_eq!(data.users[1].name, "Bob");
        assert_eq!(data.users[1].phones.len(), 1);
        assert_eq!(data.users[1].phones[0].number, "333");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("users[0].name", "Alice")
                .text("users[0].phones[0].number", "111")
                .text("users[0].phones[1].number", "222")
                .text("users[1].name", "Bob")
                .text("users[1].phones[0].number", "333"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[derive(TryFromMultipart)]
struct FormWithNestedOption {
    title: String,
    #[form_data(nested)]
    owner: Option<Person>,
}

#[tokio::test]
async fn test_nested_option_some() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedOption>| async move {
        assert_eq!(data.title, "Test Form");
        assert_eq!(data.owner, Some(Person { name: "Alice".into(), age: 30 }));
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test Form")
                .text("owner.name", "Alice")
                .text("owner.age", "30"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nested_option_none() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedOption>| async move {
        assert_eq!(data.title, "Test Form");
        assert_eq!(data.owner, None);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test Form"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
