#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart)]
struct Inner {
    required: String,
    optional: Option<i32>,
    #[form_data(default)]
    with_default: String,
    items: Vec<String>,
}

#[derive(TryFromMultipart)]
struct Middle {
    middle_field: String,
    #[form_data(flatten)]
    inner: Inner,
}

#[derive(TryFromMultipart)]
struct Outer {
    outer_field: String,
    #[form_data(flatten)]
    middle: Middle,
}

#[tokio::test]
async fn test_flatten() {
    let handler = |TypedMultipart(data): TypedMultipart<Outer>| async move {
        assert_eq!(data.outer_field, "outer");
        assert_eq!(data.middle.middle_field, "middle");
        assert_eq!(data.middle.inner.required, "req");
        assert_eq!(data.middle.inner.optional, Some(42));
        assert_eq!(data.middle.inner.with_default, "");
        assert_eq!(data.middle.inner.items, vec!["a", "b"]);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("outer_field", "outer")
                .text("middle.middle_field", "middle")
                .text("middle.inner.required", "req")
                .text("middle.inner.optional", "42")
                .text("middle.inner.items", "a")
                .text("middle.inner.items", "b"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: missing required field in nested
#[tokio::test]
async fn test_flatten_missing_required() {
    async fn handler(_: TypedMultipart<Outer>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("outer_field", "outer").text("middle.middle_field", "middle"))
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// Edge case: multiple flattened fields
#[derive(TryFromMultipart)]
struct NestedA {
    a: String,
}

#[derive(TryFromMultipart)]
struct NestedB {
    b: i32,
}

#[derive(TryFromMultipart)]
struct MultipleFlattened {
    #[form_data(flatten)]
    nested_a: NestedA,
    #[form_data(flatten)]
    nested_b: NestedB,
}

#[tokio::test]
async fn test_flatten_multiple() {
    let handler = |TypedMultipart(data): TypedMultipart<MultipleFlattened>| async move {
        assert_eq!(data.nested_a.a, "val_a");
        assert_eq!(data.nested_b.b, 123);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("nested_a.a", "val_a").text("nested_b.b", "123"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: field name collision resolved with prefix
#[derive(TryFromMultipart)]
struct NestedCollision {
    name: String,
}

#[derive(TryFromMultipart)]
struct ParentCollision {
    name: String,
    #[form_data(flatten)]
    nested: NestedCollision,
}

#[tokio::test]
async fn test_flatten_collision() {
    let handler = |TypedMultipart(data): TypedMultipart<ParentCollision>| async move {
        assert_eq!(data.name, "parent");
        assert_eq!(data.nested.name, "nested");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("name", "parent").text("nested.name", "nested"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: flatten with rename_all
#[derive(TryFromMultipart)]
#[try_from_multipart(rename_all = "UPPERCASE")]
struct NestedRenamed {
    field: String,
}

#[derive(TryFromMultipart)]
struct ParentRenamed {
    #[form_data(flatten)]
    nested: NestedRenamed,
}

#[tokio::test]
async fn test_flatten_rename_all() {
    let handler = |TypedMultipart(data): TypedMultipart<ParentRenamed>| async move {
        assert_eq!(data.nested.field, "value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("nested.FIELD", "value"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: flatten with strict mode
#[allow(dead_code)]
#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct NestedStrict {
    value: String,
}

#[allow(dead_code)]
#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct ParentStrict {
    name: String,
    #[form_data(flatten)]
    nested: NestedStrict,
}

#[tokio::test]
async fn test_flatten_strict_unknown() {
    async fn handler(_: TypedMultipart<ParentStrict>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("name", "n").text("nested.value", "v").text("unknown", "x"))
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// Edge case: custom prefix via field_name
#[derive(TryFromMultipart)]
struct NestedCustomPrefix {
    val: String,
}

#[derive(TryFromMultipart)]
struct ParentCustomPrefix {
    #[form_data(flatten, field_name = "custom")]
    nested: NestedCustomPrefix,
}

#[tokio::test]
async fn test_flatten_custom_prefix() {
    let handler = |TypedMultipart(data): TypedMultipart<ParentCustomPrefix>| async move {
        assert_eq!(data.nested.val, "test");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("custom.val", "test"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: same field name in multiple nested structs
#[derive(TryFromMultipart)]
struct NestedX {
    id: String,
}

#[derive(TryFromMultipart)]
struct NestedY {
    id: String,
}

#[derive(TryFromMultipart)]
struct ParentSameName {
    #[form_data(flatten)]
    x: NestedX,
    #[form_data(flatten)]
    y: NestedY,
}

#[tokio::test]
async fn test_flatten_same_name_different_prefix() {
    let handler = |TypedMultipart(data): TypedMultipart<ParentSameName>| async move {
        assert_eq!(data.x.id, "x_id");
        assert_eq!(data.y.id, "y_id");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("x.id", "x_id").text("y.id", "y_id"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: prefix substring (a. vs ab.)
#[derive(TryFromMultipart)]
struct NestedShort {
    v: String,
}

#[derive(TryFromMultipart)]
struct NestedLong {
    v: String,
}

#[derive(TryFromMultipart)]
struct ParentPrefixSubstring {
    #[form_data(flatten)]
    a: NestedShort,
    #[form_data(flatten)]
    ab: NestedLong,
}

#[tokio::test]
async fn test_flatten_prefix_substring() {
    let handler = |TypedMultipart(data): TypedMultipart<ParentPrefixSubstring>| async move {
        assert_eq!(data.a.v, "short");
        assert_eq!(data.ab.v, "long");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("a.v", "short").text("ab.v", "long"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: nested with all defaults (no fields provided for nested)
#[derive(TryFromMultipart)]
struct NestedDefaults {
    #[form_data(default)]
    val: String,
}

#[derive(TryFromMultipart)]
struct ParentNestedDefaults {
    name: String,
    #[form_data(flatten)]
    nested: NestedDefaults,
}

#[tokio::test]
async fn test_flatten_nested_defaults() {
    let handler = |TypedMultipart(data): TypedMultipart<ParentNestedDefaults>| async move {
        assert_eq!(data.name, "test");
        assert_eq!(data.nested.val, "");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("name", "test"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// Edge case: unknown prefixed field in strict mode
#[tokio::test]
async fn test_flatten_strict_unknown_prefixed() {
    async fn handler(_: TypedMultipart<ParentStrict>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("name", "n").text("nested.value", "v").text("nested.unknown", "x"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
