use axum::body::Bytes;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use reqwest::multipart::Form;

#[allow(dead_code)]
#[derive(TryFromMultipart)]
struct Data {
    default_limit_field: Option<Bytes>,

    #[form_data(limit = "16KiB")]
    limited_field: Option<Bytes>,

    #[form_data(limit = "unlimited")]
    unlimited_field: Option<Bytes>,
}

#[tokio::test]
async fn test_limit() {
    struct Test {
        field: &'static str,
        size: usize,
        status: StatusCode,
        error: Option<&'static str>,
    }

    let tests = [
        Test {
            field: "default_limit_field",
            size: 1024 * 1024, // 1MiB
            status: StatusCode::OK,
            error: None,
        },
        Test {
            field: "default_limit_field",
            size: 1024 * 1024 + 1, // 1.001MiB
            status: StatusCode::PAYLOAD_TOO_LARGE,
            error: Some("field 'default_limit_field' is larger than 1048576 bytes"),
        },
        Test {
            field: "limited_field",
            size: 1024 * 16, // 16KiB
            status: StatusCode::OK,
            error: None,
        },
        Test {
            field: "limited_field",
            size: 1024 * 16 + 1, // 16.001KiB
            status: StatusCode::PAYLOAD_TOO_LARGE,
            error: Some("field 'limited_field' is larger than 16384 bytes"),
        },
        Test {
            field: "unlimited_field",
            size: 1000 * 1000 * 2, // 2MB (must be lower than the axum request limit)
            status: StatusCode::OK,
            error: None,
        },
    ];

    for Test { field, size, status, error } in tests.into_iter() {
        let res =
            TestClient::new(Router::new().route("/", post(|_: TypedMultipart<Data>| async {})))
                .post("/")
                .multipart(Form::new().text(field, "x".repeat(size)))
                .send()
                .await;

        assert_eq!(res.status(), status);
        assert_eq!(res.text().await, error.unwrap_or(""));
    }
}
