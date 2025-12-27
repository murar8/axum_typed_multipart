#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[derive(TryFromMultipart, Debug, PartialEq)]
#[try_from_multipart(strict)]
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

#[tokio::test]
async fn test_nested_vec_empty() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Empty List");
        assert!(data.users.is_empty());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Empty List"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nested_vec_sparse_indices() {
    // Test that sparse indices work (e.g., [0], [5] without [1]-[4])
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Sparse List");
        // With BTreeMap, we only get the indices that were actually provided
        assert_eq!(data.users.len(), 2);
        assert_eq!(data.users[0].name, "First");
        assert_eq!(data.users[1].name, "Fifth");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Sparse List")
                .text("users[0].name", "First")
                .text("users[0].age", "1")
                .text("users[5].name", "Fifth")
                .text("users[5].age", "5"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_nested_vec_out_of_order() {
    // Test that out-of-order indices are sorted correctly
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Out of Order");
        assert_eq!(data.users.len(), 3);
        // BTreeMap ensures sorted order by index
        assert_eq!(data.users[0].name, "Zero");
        assert_eq!(data.users[1].name, "One");
        assert_eq!(data.users[2].name, "Two");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Out of Order")
                // Send in reverse order
                .text("users[2].name", "Two")
                .text("users[2].age", "2")
                .text("users[0].name", "Zero")
                .text("users[0].age", "0")
                .text("users[1].name", "One")
                .text("users[1].age", "1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// BUG #1: Prefix matching - "user" should NOT match "username"
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Inner {
    value: String,
}

#[derive(TryFromMultipart)]
struct FormWithPrefixCollision {
    #[form_data(nested)]
    user: Inner,
    username: String,
}

#[tokio::test]
async fn test_prefix_no_false_match() {
    // BUG: "user" prefix should NOT match "username" field
    // The field "username" should go to the simple `username` field, not to nested `user`
    let handler = |TypedMultipart(data): TypedMultipart<FormWithPrefixCollision>| async move {
        assert_eq!(data.user.value, "nested_value");
        assert_eq!(data.username, "simple_value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("user.value", "nested_value").text("username", "simple_value"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// This test triggers the actual bug: nested struct has a field that matches
// the "leftover" after strip_prefix
#[derive(TryFromMultipart, Debug, PartialEq)]
struct InnerWithName {
    name: String, // "username" - "user" = "name" - COLLISION!
}

#[derive(TryFromMultipart)]
struct FormWithActualPrefixBug {
    #[form_data(nested)]
    user: InnerWithName,
    username: String,
}

#[tokio::test]
async fn test_prefix_collision_with_nested_field() {
    // This test exposes the prefix bug:
    // - "username" is a simple field
    // - But strip_prefix("user") on "username" returns "name"
    // - Nested InnerWithName has a "name" field
    // - So "username" value gets incorrectly consumed as "user.name"!
    let handler = |TypedMultipart(data): TypedMultipart<FormWithActualPrefixBug>| async move {
        // If bug exists: user.name = "simple_value", username = missing
        // Expected: user.name = "nested_value", username = "simple_value"
        assert_eq!(data.user.name, "nested_value");
        assert_eq!(data.username, "simple_value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("user.name", "nested_value").text("username", "simple_value"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// BUG #2: Option<Vec<T>> should preserve the Option wrapper
// =============================================================================

#[derive(TryFromMultipart)]
struct FormWithOptionalVec {
    title: String,
    #[form_data(nested)]
    items: Option<Vec<Inner>>,
}

#[tokio::test]
async fn test_option_vec_none() {
    // When no items are provided, Option<Vec<Inner>> should be None, not Some([])
    let handler = |TypedMultipart(data): TypedMultipart<FormWithOptionalVec>| async move {
        assert_eq!(data.title, "No Items");
        assert_eq!(data.items, None);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "No Items"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_option_vec_some() {
    // When items are provided, Option<Vec<Inner>> should be Some(vec![...])
    let handler = |TypedMultipart(data): TypedMultipart<FormWithOptionalVec>| async move {
        assert_eq!(data.title, "Has Items");
        assert_eq!(
            data.items,
            Some(vec![Inner { value: "a".into() }, Inner { value: "b".into() }])
        );
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Has Items")
                .text("items[0].value", "a")
                .text("items[1].value", "b"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Invalid index formats - should be rejected gracefully
// =============================================================================

#[tokio::test]
async fn test_invalid_index_negative() {
    // Negative index should be rejected (field ignored in non-strict mode)
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Negative Index");
        // Negative index field should be ignored, only valid index processed
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.users[0].name, "Valid");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Negative Index")
                .text("users[-1].name", "Invalid")
                .text("users[-1].age", "0")
                .text("users[0].name", "Valid")
                .text("users[0].age", "1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_index_non_numeric() {
    // Non-numeric index should be rejected (field ignored in non-strict mode)
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Non-numeric Index");
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.users[0].name, "Valid");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Non-numeric Index")
                .text("users[abc].name", "Invalid")
                .text("users[abc].age", "0")
                .text("users[0].name", "Valid")
                .text("users[0].age", "1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_index_empty_brackets() {
    // Empty brackets should be rejected
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Empty Brackets");
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.users[0].name, "Valid");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Empty Brackets")
                .text("users[].name", "Invalid")
                .text("users[].age", "0")
                .text("users[0].name", "Valid")
                .text("users[0].age", "1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Missing required nested fields - error cases
// =============================================================================

#[tokio::test]
async fn test_missing_required_nested_field() {
    // Missing required field in nested struct should fail
    let handler = |TypedMultipart(_data): TypedMultipart<FormWithNestedSingle>| async move {};

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("title", "Test").text("owner.name", "Alice"),
            // Missing owner.age!
        )
        .await;

    // Should fail because owner.age is missing
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_entire_required_nested() {
    // Missing entire required nested struct should fail
    let handler = |TypedMultipart(_data): TypedMultipart<FormWithNestedSingle>| async move {};

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test"))
        // Missing owner entirely!
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// =============================================================================
// Three levels of nesting
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Level3 {
    data: String,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Level2 {
    name: String,
    #[form_data(nested)]
    items: Vec<Level3>,
}

#[derive(TryFromMultipart)]
struct Level1 {
    #[form_data(nested)]
    groups: Vec<Level2>,
}

#[tokio::test]
async fn test_three_levels_of_nesting() {
    let handler = |TypedMultipart(data): TypedMultipart<Level1>| async move {
        assert_eq!(data.groups.len(), 2);
        assert_eq!(data.groups[0].name, "Group A");
        assert_eq!(data.groups[0].items.len(), 2);
        assert_eq!(data.groups[0].items[0].data, "A1");
        assert_eq!(data.groups[0].items[1].data, "A2");
        assert_eq!(data.groups[1].name, "Group B");
        assert_eq!(data.groups[1].items.len(), 1);
        assert_eq!(data.groups[1].items[0].data, "B1");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("groups[0].name", "Group A")
                .text("groups[0].items[0].data", "A1")
                .text("groups[0].items[1].data", "A2")
                .text("groups[1].name", "Group B")
                .text("groups[1].items[0].data", "B1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Nested with field_name rename
// =============================================================================

#[derive(TryFromMultipart)]
struct FormWithRenamedNested {
    #[form_data(nested, field_name = "p")]
    person: Person,
}

#[tokio::test]
async fn test_nested_with_field_name_rename() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithRenamedNested>| async move {
        assert_eq!(data.person.name, "Alice");
        assert_eq!(data.person.age, 30);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("p.name", "Alice").text("p.age", "30"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Nested with default attribute
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq, Default)]
struct DefaultableInner {
    #[form_data(default)]
    value: String,
}

#[derive(TryFromMultipart)]
struct FormWithDefaultNested {
    title: String,
    #[form_data(nested, default)]
    optional: DefaultableInner,
}

#[tokio::test]
async fn test_nested_with_default() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithDefaultNested>| async move {
        assert_eq!(data.title, "Test");
        // When nested field is missing but has default, should use default
        assert_eq!(data.optional, DefaultableInner::default());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Vec<Option<T>> - sparse optional elements
// =============================================================================

#[derive(TryFromMultipart)]
struct FormWithVecOption {
    title: String,
    #[form_data(nested)]
    items: Vec<Option<Inner>>,
}

#[tokio::test]
async fn test_vec_option_nested() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithVecOption>| async move {
        assert_eq!(data.title, "Test");
        assert_eq!(data.items.len(), 2);
        assert_eq!(data.items[0], Some(Inner { value: "a".into() }));
        assert_eq!(data.items[1], Some(Inner { value: "b".into() }));
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("items[0].value", "a")
                .text("items[1].value", "b"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Strict mode with nested structs - duplicate detection
// =============================================================================

#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct StrictFormWithNested {
    #[form_data(nested)]
    owner: Person,
}

#[tokio::test]
async fn test_strict_nested_duplicate_field() {
    // In strict mode, sending the same nested field twice should fail
    let handler = |TypedMultipart(data): TypedMultipart<StrictFormWithNested>| async move {
        let _ = data.owner; // Silence unused warning
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("owner.name", "Alice")
                .text("owner.name", "Bob") // Duplicate!
                .text("owner.age", "30"),
        )
        .await;

    // Should fail with duplicate field error
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// =============================================================================
// Whitespace in brackets should be rejected
// =============================================================================

#[tokio::test]
async fn test_whitespace_in_brackets_rejected() {
    // Whitespace in brackets like [  0  ] should be rejected
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Test");
        // Only the valid index should be parsed
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.users[0].name, "Valid");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("users[  0  ].name", "Invalid") // Whitespace - should be rejected
                .text("users[  0  ].age", "0")
                .text("users[0].name", "Valid")
                .text("users[0].age", "1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
