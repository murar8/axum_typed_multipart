//! NOTE: This test suite was largely AI-generated. The tests are functional and pass,
//! but may not be worth in-depth human review. Focus review efforts elsewhere.

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
    // Missing required field in nested struct should fail with full path in error
    let handler = |TypedMultipart(_data): TypedMultipart<FormWithNestedSingle>| async move {};

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("title", "Test").text("owner.name", "Alice"),
            // Missing owner.age!
        )
        .await;

    // Should fail because owner.age is missing - error should show full path
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'owner.age' is required");
}

#[tokio::test]
async fn test_missing_entire_required_nested() {
    // Missing entire required nested struct should fail with full path
    let handler = |TypedMultipart(_data): TypedMultipart<FormWithNestedSingle>| async move {};

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test"))
        // Missing owner entirely!
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    // First missing field (alphabetically) in nested struct should show full path
    assert_eq!(res.text().await, "field 'owner.age' is required");
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
// Indirect Vec/Option - nested struct with Vec/Option simple fields
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct PersonWithTags {
    name: String,
    tags: Vec<String>,
    nickname: Option<String>,
}

#[derive(TryFromMultipart)]
struct FormWithIndirectVecOption {
    #[form_data(nested)]
    person: PersonWithTags,
}

#[tokio::test]
async fn test_indirect_vec_in_nested() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithIndirectVecOption>| async move {
        assert_eq!(data.person.name, "Alice");
        assert_eq!(data.person.tags, vec!["admin", "user"]);
        assert_eq!(data.person.nickname, Some("Ali".to_string()));
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("person.name", "Alice")
                .text("person.tags", "admin")
                .text("person.tags", "user")
                .text("person.nickname", "Ali"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_indirect_option_none_in_nested() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithIndirectVecOption>| async move {
        assert_eq!(data.person.name, "Bob");
        assert!(data.person.tags.is_empty());
        assert_eq!(data.person.nickname, None);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("person.name", "Bob"))
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

// =============================================================================
// Strict mode with nested Vec - invalid indices should be rejected
// =============================================================================

#[allow(dead_code)]
#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct StrictFormWithNestedVec {
    title: String,
    #[form_data(nested)]
    users: Vec<Person>,
}

#[tokio::test]
async fn test_strict_invalid_index_non_numeric() {
    // In strict mode, non-numeric index should be rejected as unknown field
    async fn handler(_: TypedMultipart<StrictFormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("users[abc].name", "Invalid")
                .text("users[abc].age", "0"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'users[abc].name' is not expected");
}

#[tokio::test]
async fn test_strict_invalid_index_negative() {
    // In strict mode, negative index should be rejected as unknown field
    async fn handler(_: TypedMultipart<StrictFormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("users[-1].name", "Invalid")
                .text("users[-1].age", "0"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'users[-1].name' is not expected");
}

#[tokio::test]
async fn test_strict_invalid_index_whitespace() {
    // In strict mode, whitespace in brackets should be rejected
    // Note: reqwest converts spaces to empty field name in multipart encoding
    async fn handler(_: TypedMultipart<StrictFormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("users[  0  ].name", "Invalid")
                .text("users[  0  ].age", "0"),
        )
        .await;

    // Due to multipart encoding quirks, field name with spaces becomes empty
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_strict_invalid_index_empty_brackets() {
    // In strict mode, empty brackets should be rejected
    async fn handler(_: TypedMultipart<StrictFormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("users[].name", "Invalid")
                .text("users[].age", "0"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'users[].name' is not expected");
}

// =============================================================================
// 4+ levels of deep nesting
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Level4 {
    data: String,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Level3Item {
    label: String,
    #[form_data(nested)]
    entries: Vec<Level4>,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Level2Section {
    name: String,
    #[form_data(nested)]
    items: Vec<Level3Item>,
}

#[derive(TryFromMultipart)]
struct Level1Form {
    title: String,
    #[form_data(nested)]
    sections: Vec<Level2Section>,
}

#[tokio::test]
async fn test_four_levels_of_nesting() {
    let handler = |TypedMultipart(data): TypedMultipart<Level1Form>| async move {
        assert_eq!(data.title, "Deep");
        assert_eq!(data.sections.len(), 2);
        assert_eq!(data.sections[0].name, "S0");
        assert_eq!(data.sections[0].items.len(), 2);
        assert_eq!(data.sections[0].items[0].label, "I0");
        assert_eq!(data.sections[0].items[0].entries.len(), 2);
        assert_eq!(data.sections[0].items[0].entries[0].data, "E0");
        assert_eq!(data.sections[0].items[0].entries[1].data, "E1");
        assert_eq!(data.sections[0].items[1].label, "I1");
        assert_eq!(data.sections[1].name, "S1");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Deep")
                .text("sections[0].name", "S0")
                .text("sections[0].items[0].label", "I0")
                .text("sections[0].items[0].entries[0].data", "E0")
                .text("sections[0].items[0].entries[1].data", "E1")
                .text("sections[0].items[1].label", "I1")
                .text("sections[1].name", "S1"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deep_sparse_indices() {
    // Sparse indices at every level should be normalized
    let handler = |TypedMultipart(data): TypedMultipart<Level1Form>| async move {
        assert_eq!(data.sections.len(), 2);
        assert_eq!(data.sections[0].name, "Sparse0");
        assert_eq!(data.sections[0].items[0].label, "Item5");
        assert_eq!(data.sections[0].items[0].entries[0].data, "Entry99");
        assert_eq!(data.sections[1].name, "Sparse10");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Sparse")
                .text("sections[0].name", "Sparse0")
                .text("sections[0].items[5].label", "Item5")
                .text("sections[0].items[5].entries[99].data", "Entry99")
                .text("sections[10].name", "Sparse10"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_deep_out_of_order() {
    // Fields sent in completely reversed order
    let handler = |TypedMultipart(data): TypedMultipart<Level1Form>| async move {
        assert_eq!(data.title, "Reversed");
        assert_eq!(data.sections[0].name, "First");
        assert_eq!(data.sections[0].items[0].label, "Item");
        assert_eq!(data.sections[0].items[0].entries[0].data, "Deep");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                // Send deepest first, title last
                .text("sections[0].items[0].entries[0].data", "Deep")
                .text("sections[0].items[0].label", "Item")
                .text("sections[0].name", "First")
                .text("title", "Reversed"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Multiple Vec fields at same level
// =============================================================================

#[derive(TryFromMultipart)]
struct MultiVecForm {
    #[form_data(nested)]
    users: Vec<Inner>,
    #[form_data(nested)]
    admins: Vec<Inner>,
    #[form_data(nested)]
    guests: Vec<Inner>,
}

#[tokio::test]
async fn test_multiple_vec_fields() {
    let handler = |TypedMultipart(data): TypedMultipart<MultiVecForm>| async move {
        assert_eq!(data.users.len(), 2);
        assert_eq!(data.users[0].value, "User1");
        assert_eq!(data.users[1].value, "User2");
        assert_eq!(data.admins.len(), 1);
        assert_eq!(data.admins[0].value, "Admin1");
        assert_eq!(data.guests.len(), 3);
        assert_eq!(data.guests[2].value, "Guest3");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("users[0].value", "User1")
                .text("admins[0].value", "Admin1")
                .text("guests[0].value", "Guest1")
                .text("users[1].value", "User2")
                .text("guests[1].value", "Guest2")
                .text("guests[2].value", "Guest3"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_multiple_vec_some_empty() {
    let handler = |TypedMultipart(data): TypedMultipart<MultiVecForm>| async move {
        assert_eq!(data.users.len(), 1);
        assert!(data.admins.is_empty());
        assert!(data.guests.is_empty());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("users[0].value", "OnlyUser"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Unicode and special characters
// =============================================================================

#[tokio::test]
async fn test_unicode_values() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedSingle>| async move {
        assert_eq!(data.title, "日本語");
        assert_eq!(data.owner.name, "Алексей");
        assert_eq!(data.owner.age, 25);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "日本語")
                .text("owner.name", "Алексей")
                .text("owner.age", "25"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Duplicate fields at nested levels (non-strict: last wins)
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct LaxPerson {
    name: String,
    age: u32,
}

#[allow(dead_code)]
#[derive(TryFromMultipart)]
struct LaxFormWithNestedVec {
    title: String,
    #[form_data(nested)]
    users: Vec<LaxPerson>,
}

#[tokio::test]
async fn test_duplicate_nested_last_wins() {
    // In non-strict mode, last value should win
    let handler = |TypedMultipart(data): TypedMultipart<LaxFormWithNestedVec>| async move {
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.users[0].name, "Second"); // Last wins
        assert_eq!(data.users[0].age, 99);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Dups")
                .text("users[0].name", "First")
                .text("users[0].age", "1")
                .text("users[0].name", "Second") // Overwrites
                .text("users[0].age", "99"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Error message path tests
// =============================================================================

#[tokio::test]
async fn test_error_path_vec_item() {
    // Missing field in Vec item should show full path with index
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("title", "Test").text("users[0].name", "Alice"),
            // Missing users[0].age!
        )
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'users[0].age' is required");
}

#[tokio::test]
async fn test_error_path_deep_nesting() {
    // Vec fields are optional (default to empty), so this should succeed
    let handler = |TypedMultipart(data): TypedMultipart<Level1Form>| async move {
        assert_eq!(data.title, "Test");
        assert_eq!(data.sections[0].name, "Section");
        assert!(data.sections[0].items.is_empty());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test").text("sections[0].name", "Section"))
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_error_path_deep_nesting_required() {
    // Missing required field deep in nesting should show full path
    async fn handler(_: TypedMultipart<Level1Form>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("title", "Test").text("sections[0].items[0].label", "Item"),
            // Missing sections[0].name!
        )
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await, "field 'sections[0].name' is required");
}

#[tokio::test]
async fn test_error_path_four_levels() {
    // Vec fields are optional (default to empty), so this should succeed
    let handler = |TypedMultipart(data): TypedMultipart<Level1Form>| async move {
        assert_eq!(data.title, "Test");
        assert_eq!(data.sections[0].name, "Section");
        assert_eq!(data.sections[0].items[0].label, "Item");
        assert!(data.sections[0].items[0].entries.is_empty());
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("sections[0].name", "Section")
                .text("sections[0].items[0].label", "Item"),
        )
        .await;

    assert_eq!(res.status(), StatusCode::OK);
}
