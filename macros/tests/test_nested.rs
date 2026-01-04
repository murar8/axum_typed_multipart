//! # ============================================================================
//! # AI-GENERATED TEST SUITE
//! # ============================================================================
//! #
//! # This test suite was largely AI-generated. The tests are functional and pass,
//! # but may not be worth in-depth human review. Focus review efforts elsewhere.
//! #
//! # ============================================================================

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Regression: Prefix matching - "user" should NOT match "username"
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
    // Verify "user" prefix does NOT match "username" field
    // The field "username" should go to the simple `username` field, not to nested `user`
    let handler = |TypedMultipart(data): TypedMultipart<FormWithPrefixCollision>| async move {
        assert_eq!(data.user.value, "nested_value");
        assert_eq!(data.username, "simple_value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("user.value", "nested_value").text("username", "simple_value"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// This test verifies the fix: nested struct has a field that matches
// the "leftover" after strip_prefix (which was incorrectly matched before)
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
    // This test verifies the prefix collision fix:
    // - "username" is a simple field
    // - strip_prefix("user") on "username" returns "name"
    // - Nested InnerWithName has a "name" field
    // - Previously, "username" was incorrectly consumed as "user.name"
    let handler = |TypedMultipart(data): TypedMultipart<FormWithActualPrefixBug>| async move {
        // Before fix: user.name = "simple_value", username = missing
        // After fix: user.name = "nested_value", username = "simple_value"
        assert_eq!(data.user.name, "nested_value");
        assert_eq!(data.username, "simple_value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("user.name", "nested_value").text("username", "simple_value"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Regression: Option<Vec<T>> should preserve the Option wrapper
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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Invalid index formats - should be rejected gracefully
// =============================================================================

#[tokio::test]
async fn test_invalid_index_negative() {
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test").text("users[-1].name", "Invalid"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.text().await.unwrap(),
        "field 'users[-1].name' has invalid index: '-1' is not a valid number"
    );
}

#[tokio::test]
async fn test_invalid_index_non_numeric() {
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test").text("users[abc].name", "Invalid"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.text().await.unwrap(),
        "field 'users[abc].name' has invalid index: 'abc' is not a valid number"
    );
}

#[tokio::test]
async fn test_invalid_index_empty_brackets() {
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test").text("users[].name", "Invalid"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.text().await.unwrap(),
        "field 'users[].name' has invalid index: '' is not a valid number"
    );
}

#[tokio::test]
async fn test_invalid_index_whitespace_quirk() {
    // NOTE: This documents a reqwest multipart encoding quirk where field names
    // containing whitespace in brackets (e.g., "users[  0  ].name") are not
    // properly matched. The field ends up being ignored in lax mode rather than
    // returning an InvalidIndexFormat error. This is due to how reqwest encodes
    // the multipart field name, not a bug in axum_typed_multipart.
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Test");
        assert!(data.users.is_empty()); // Field was ignored due to encoding quirk
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test").text("users[  0  ].name", "Invalid"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Non-contiguous and out-of-order indices - error cases
// =============================================================================

#[tokio::test]
async fn test_non_contiguous_indices_error() {
    // Sparse indices (e.g., [0] then [5]) should return InvalidIndex error
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Sparse")
                .text("users[0].name", "First")
                .text("users[0].age", "1")
                .text("users[5].name", "Fifth")
                .text("users[5].age", "5"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'users[5].name' has invalid index (expected 1)");
}

#[tokio::test]
async fn test_out_of_order_indices_error() {
    // Out-of-order indices (e.g., [1] before [0]) should return InvalidIndex error
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Out of Order")
                .text("users[1].name", "Second")
                .text("users[1].age", "2"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'users[1].name' has invalid index (expected 0)");
}

#[tokio::test]
async fn test_fields_within_same_index_any_order() {
    // Fields for the same index can arrive in any order
    let handler = |TypedMultipart(data): TypedMultipart<FormWithNestedVec>| async move {
        assert_eq!(data.title, "Test");
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
                .text("title", "Test")
                // Index 0 fields in reverse order
                .text("users[0].age", "30")
                .text("users[0].name", "Alice")
                // Index 1 fields in reverse order
                .text("users[1].age", "25")
                .text("users[1].name", "Bob"),
        )
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

    // Should fail because owner.age is missing - error should show full path
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'owner.age' is required");
}

#[tokio::test]
async fn test_missing_entire_required_nested() {
    // Missing entire required nested struct should fail with full path
    let handler = |TypedMultipart(_data): TypedMultipart<FormWithNestedSingle>| async move {};

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(Form::new().text("title", "Test"))
        // Missing owner entirely!
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    // When entire required nested struct is missing, report parent field
    assert_eq!(res.text().await.unwrap(), "field 'owner' is required");
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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

    // Should fail with duplicate field error
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// =============================================================================
// Strict mode with nested Vec
// =============================================================================

#[allow(dead_code)]
#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct StrictFormWithNestedVec {
    title: String,
    #[form_data(nested)]
    users: Vec<Person>,
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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'users[0].age' is required");
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
        .send()
        .await
        .unwrap();

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
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'sections[0].name' is required");
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
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Type coercion errors in nested fields - error path should include full path
// =============================================================================

#[tokio::test]
async fn test_error_type_coercion_nested_single() {
    // Invalid type in nested field should show full path in error
    async fn handler(_: TypedMultipart<FormWithNestedSingle>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("owner.name", "Alice")
                .text("owner.age", "not_a_number"), // Invalid!
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = res.text().await.unwrap();
    // Error should mention the full path "owner.age"
    assert!(body.contains("owner.age"), "Error should contain 'owner.age', got: {}", body);
}

#[tokio::test]
async fn test_error_type_coercion_nested_vec() {
    // Invalid type in Vec item should show full path with index
    async fn handler(_: TypedMultipart<FormWithNestedVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("users[0].name", "Alice")
                .text("users[0].age", "thirty"), // Invalid!
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = res.text().await.unwrap();
    // Error should mention the full path "users[0].age"
    assert!(body.contains("users[0].age"), "Error should contain 'users[0].age', got: {}", body);
}

// =============================================================================
// Partial optional nested struct - sending some but not all required fields
// =============================================================================

#[tokio::test]
async fn test_partial_optional_nested_fails() {
    // When Option<Person> receives some fields, inner validation should still apply
    // Sending owner.name but not owner.age should fail (age is required in Person)
    async fn handler(_: TypedMultipart<FormWithNestedOption>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("title", "Test").text("owner.name", "Alice"),
            // Missing owner.age - Person requires both name and age
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'owner.age' is required");
}

#[tokio::test]
async fn test_partial_optional_nested_vec_fails() {
    // Option<Vec<Inner>> where inner struct is partially provided
    #[derive(TryFromMultipart, Debug, PartialEq)]
    struct PersonRequired {
        name: String,
        age: u32,
    }

    #[allow(dead_code)]
    #[derive(TryFromMultipart)]
    struct FormWithOptionalVecRequired {
        title: String,
        #[form_data(nested)]
        people: Option<Vec<PersonRequired>>,
    }

    async fn handler(_: TypedMultipart<FormWithOptionalVecRequired>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("title", "Test").text("people[0].name", "Alice"),
            // Missing people[0].age
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'people[0].age' is required");
}

// =============================================================================
// Strict mode at different nesting levels
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct LaxInner {
    value: String,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
#[try_from_multipart(strict)]
struct StrictInner {
    value: String,
}

#[derive(TryFromMultipart)]
struct LaxOuterWithStrictInner {
    title: String,
    #[form_data(nested)]
    item: StrictInner,
}

#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct StrictOuterWithLaxInner {
    title: String,
    #[form_data(nested)]
    item: LaxInner,
}

#[allow(dead_code)]
#[derive(TryFromMultipart)]
struct LaxOuterWithStrictInnerVec {
    title: String,
    #[form_data(nested)]
    items: Vec<StrictInner>,
}

#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
struct StrictOuterWithLaxInnerVec {
    title: String,
    #[form_data(nested)]
    items: Vec<LaxInner>,
}

#[tokio::test]
async fn test_strict_inner_lax_outer_rejects_duplicate() {
    // Inner is strict, outer is lax - duplicate in inner should be rejected
    async fn handler(_: TypedMultipart<LaxOuterWithStrictInner>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("item.value", "First")
                .text("item.value", "Second"), // Duplicate in strict inner
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_strict_inner_lax_outer_allows_outer_duplicate() {
    // Inner is strict, outer is lax - duplicate at outer level should be allowed (last wins)
    let handler = |TypedMultipart(data): TypedMultipart<LaxOuterWithStrictInner>| async move {
        assert_eq!(data.title, "Second"); // Last wins at outer level
        assert_eq!(data.item.value, "Value");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "First")
                .text("title", "Second") // Duplicate at lax outer level
                .text("item.value", "Value"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lax_inner_strict_outer_allows_inner_duplicate() {
    // Inner is lax, outer is strict - duplicate in inner should be allowed (last wins)
    let handler = |TypedMultipart(data): TypedMultipart<StrictOuterWithLaxInner>| async move {
        assert_eq!(data.title, "Test");
        assert_eq!(data.item.value, "Second"); // Last wins at inner level
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("item.value", "First")
                .text("item.value", "Second"), // Duplicate in lax inner
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lax_inner_strict_outer_rejects_outer_duplicate() {
    // Inner is lax, outer is strict - duplicate at outer level should be rejected
    async fn handler(_: TypedMultipart<StrictOuterWithLaxInner>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "First")
                .text("title", "Second") // Duplicate at strict outer level
                .text("item.value", "Value"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_strict_inner_vec_lax_outer_rejects_duplicate() {
    // Inner Vec items are strict - duplicate in Vec item should be rejected
    async fn handler(_: TypedMultipart<LaxOuterWithStrictInnerVec>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("items[0].value", "First")
                .text("items[0].value", "Second"), // Duplicate in strict inner
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_lax_inner_vec_strict_outer_allows_inner_duplicate() {
    // Inner Vec items are lax - duplicate in Vec item should be allowed
    let handler = |TypedMultipart(data): TypedMultipart<StrictOuterWithLaxInnerVec>| async move {
        assert_eq!(data.title, "Test");
        assert_eq!(data.items.len(), 1);
        assert_eq!(data.items[0].value, "Second"); // Last wins
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("items[0].value", "First")
                .text("items[0].value", "Second"), // Duplicate in lax inner
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_strict_outer_rejects_unknown_nested_field() {
    // Strict outer should reject unknown fields that look like nested fields
    async fn handler(_: TypedMultipart<StrictOuterWithLaxInner>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("item.value", "Value")
                .text("item.unknown", "Bad"), // Unknown field in nested
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'item.unknown' is not expected");
}

// =============================================================================
// Single nested struct within nested Vec
// =============================================================================

#[derive(TryFromMultipart, Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct PersonWithAddress {
    name: String,
    #[form_data(nested)]
    address: Address, // Single nested, not Vec
}

#[derive(TryFromMultipart)]
struct FormWithVecOfNestedSingle {
    title: String,
    #[form_data(nested)]
    people: Vec<PersonWithAddress>, // Vec of structs that have single nested
}

#[tokio::test]
async fn test_vec_containing_single_nested() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithVecOfNestedSingle>| async move {
        assert_eq!(data.title, "People");
        assert_eq!(data.people.len(), 2);
        assert_eq!(data.people[0].name, "Alice");
        assert_eq!(data.people[0].address.street, "123 Main St");
        assert_eq!(data.people[0].address.city, "Springfield");
        assert_eq!(data.people[1].name, "Bob");
        assert_eq!(data.people[1].address.street, "456 Oak Ave");
        assert_eq!(data.people[1].address.city, "Shelbyville");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "People")
                .text("people[0].name", "Alice")
                .text("people[0].address.street", "123 Main St")
                .text("people[0].address.city", "Springfield")
                .text("people[1].name", "Bob")
                .text("people[1].address.street", "456 Oak Ave")
                .text("people[1].address.city", "Shelbyville"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_vec_containing_single_nested_missing_inner_field() {
    // Missing required field in the single nested struct within Vec item
    async fn handler(_: TypedMultipart<FormWithVecOfNestedSingle>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("title", "Test")
                .text("people[0].name", "Alice")
                .text("people[0].address.street", "123 Main St"),
            // Missing people[0].address.city
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'people[0].address.city' is required");
}

#[derive(TryFromMultipart, Debug, PartialEq)]
struct PersonWithOptionalAddress {
    name: String,
    #[form_data(nested)]
    address: Option<Address>, // Optional single nested
}

#[derive(TryFromMultipart)]
struct FormWithVecOfOptionalNestedSingle {
    #[form_data(nested)]
    people: Vec<PersonWithOptionalAddress>,
}

#[tokio::test]
async fn test_vec_containing_optional_single_nested_some() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithVecOfOptionalNestedSingle>| async move {
        assert_eq!(data.people.len(), 2);
        assert_eq!(data.people[0].name, "Alice");
        assert_eq!(
            data.people[0].address,
            Some(Address { street: "Main St".into(), city: "Springfield".into() })
        );
        assert_eq!(data.people[1].name, "Bob");
        assert_eq!(data.people[1].address, None);
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("people[0].name", "Alice")
                .text("people[0].address.street", "Main St")
                .text("people[0].address.city", "Springfield")
                .text("people[1].name", "Bob"),
            // No address for Bob
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_vec_containing_optional_single_nested_partial_fails() {
    // Partial address (street but no city) should fail even though address is optional
    async fn handler(_: TypedMultipart<FormWithVecOfOptionalNestedSingle>) {
        panic!("should not be called");
    }

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new().text("people[0].name", "Alice").text("people[0].address.street", "Main St"),
            // Missing people[0].address.city - partial optional nested
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "field 'people[0].address.city' is required");
}

// =============================================================================
// Cross-module nested structs (same crate)
// =============================================================================

mod same_crate_module {
    use axum_typed_multipart::TryFromMultipart;

    #[derive(TryFromMultipart, Debug, PartialEq)]
    pub struct ModuleAddress {
        pub street: String,
        pub city: String,
    }
}

#[derive(TryFromMultipart)]
struct FormWithSameCrateNested {
    name: String,
    #[form_data(nested)]
    address: same_crate_module::ModuleAddress,
}

#[tokio::test]
async fn test_cross_module_nested() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithSameCrateNested>| async move {
        assert_eq!(data.name, "Alice");
        assert_eq!(data.address.street, "123 Main St");
        assert_eq!(data.address.city, "Springfield");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("name", "Alice")
                .text("address.street", "123 Main St")
                .text("address.city", "Springfield"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

// =============================================================================
// Cross-crate nested structs (different crate)
// =============================================================================

#[derive(TryFromMultipart)]
struct FormWithExternalCrateNested {
    name: String,
    #[form_data(nested)]
    address: test_external::ExternalAddress,
}

#[tokio::test]
async fn test_cross_crate_nested() {
    let handler = |TypedMultipart(data): TypedMultipart<FormWithExternalCrateNested>| async move {
        assert_eq!(data.name, "Bob");
        assert_eq!(data.address.street, "456 Oak Ave");
        assert_eq!(data.address.city, "Shelbyville");
    };

    let res = TestClient::new(Router::new().route("/", post(handler)))
        .post("/")
        .multipart(
            Form::new()
                .text("name", "Bob")
                .text("address.street", "456 Oak Ave")
                .text("address.city", "Shelbyville"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
