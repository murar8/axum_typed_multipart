#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

use axum::http::StatusCode;
use axum_test_helper::TestClient;
use reqwest::multipart::{Form, Part};

#[path = "../examples/basic.rs"]
#[allow(dead_code)]
mod basic;

#[path = "../examples/custom_error.rs"]
#[allow(dead_code)]
mod custom_error;

#[path = "../examples/state.rs"]
#[allow(dead_code)]
mod state;

#[path = "../examples/type_safe_enums.rs"]
#[allow(dead_code)]
mod type_safe_enums;

#[path = "../examples/upload.rs"]
#[allow(dead_code)]
mod upload;

#[path = "../examples/utoipa.rs"]
#[allow(dead_code)]
mod utoipa_example;

// basic.rs tests

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_basic_create_user_success() {
    let client = TestClient::new(basic::app());
    let form = Form::new().text("first_name", "John").text("last_name", "Doe");
    let res = client.post("/users/create").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_basic_create_user_missing_field() {
    let client = TestClient::new(basic::app());
    let form = Form::new().text("first_name", "John");
    let res = client.post("/users/create").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// custom_error.rs tests

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_custom_error_success() {
    let client = TestClient::new(custom_error::app());
    let form = Form::new().text("name", "item1").text("position", "42");
    let res = client.post("/position/update").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_custom_error_missing_field() {
    let client = TestClient::new(custom_error::app());
    let form = Form::new().text("name", "item1");
    let res = client.post("/position/update").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// state.rs tests

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_state_valid_role() {
    let client = TestClient::new(state::app());
    let form = Form::new().text("role", "admin");
    let res = client.post("/user/update").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_state_invalid_role() {
    let client = TestClient::new(state::app());
    let form = Form::new().text("role", "superuser");
    let res = client.post("/user/update").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// type_safe_enums.rs tests

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_enum_valid() {
    let client = TestClient::new(type_safe_enums::app());
    let form = Form::new().text("name", "John").text("sex", "Male");
    let res = client.post("/").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_enum_invalid() {
    let client = TestClient::new(type_safe_enums::app());
    let form = Form::new().text("name", "Alex").text("sex", "Invalid");
    let res = client.post("/").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// upload.rs tests

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_upload_success() {
    let client = TestClient::new(upload::app());
    let file_part = Part::bytes(b"test content".to_vec()).file_name("test.txt");
    let form = Form::new().part("image", file_part).text("author", "testuser");
    let res = client.post("/").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_upload_missing_file() {
    let client = TestClient::new(upload::app());
    let form = Form::new().text("author", "testuser");
    let res = client.post("/").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// utoipa.rs tests

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_utoipa_upload() {
    let client = TestClient::new(utoipa_example::app());
    let file = Part::bytes(b"hello".to_vec()).file_name("test.txt");
    let form = Form::new().text("name", "John").part("file", file);
    let res = client.post("/upload").multipart(form).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
async fn test_utoipa_openapi_schema() {
    let client = TestClient::new(utoipa_example::app());
    let res = client.get("/api-docs/openapi2.json").send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
