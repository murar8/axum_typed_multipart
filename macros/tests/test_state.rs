#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
use axum::extract::multipart::Field;
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_test_helper::TestClient;
use axum_typed_multipart::{
    anyhow, async_trait, FieldData, TryFromField as _, TryFromFieldWithState, TryFromMultipart,
    TypedMultipart, TypedMultipartError,
};
use reqwest::multipart::Form;

#[derive(Clone)]
struct AppState<T> {
    state_data: T,
}

struct Position(u32);

#[async_trait]
impl<T> TryFromFieldWithState<AppState<T>> for Position
where
    T: Into<u32> + Sync + Copy,
{
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        state: &AppState<T>,
    ) -> Result<Self, TypedMultipartError> {
        let position = u32::try_from_field(field, limit_bytes).await?;
        if position >= state.state_data.into() {
            Ok(Position(position))
        } else {
            Err(TypedMultipartError::Other {
                source: anyhow::anyhow!(
                    "position must be greater than or equal to {}",
                    state.state_data.into()
                ),
            })
        }
    }
}

#[derive(TryFromMultipart)]
#[try_from_multipart(strict, state = AppState::<u32>)]
struct Data {
    name: FieldData<String>,
    items: Vec<String>,
    #[form_data(default)]
    default_value: String,
    position: FieldData<Position>,
}

fn create_test_client<H, T, S>(handler: H, state: S) -> TestClient
where
    H: Handler<T, AppState<S>>,
    S: Clone + Sync + Send + 'static,
    T: Send + Sync + 'static,
{
    TestClient::new(
        Router::new().route("/", post(handler)).with_state(AppState { state_data: state }),
    )
}

#[tokio::test]
async fn test_state_valid() {
    async fn handler(TypedMultipart(data): TypedMultipart<Data>) {
        assert_eq!(data.name.contents, "data");
        assert_eq!(data.items, vec!["bread", "cheese"]);
        assert_eq!(data.default_value, "default");
        assert_eq!(data.position.contents.0, 20);
    }
    let form = Form::new()
        .text("name", "data")
        .text("items", "bread")
        .text("items", "cheese")
        .text("default_value", "default")
        .text("position", "20");
    let res = create_test_client(handler, 0u32).post("/").multipart(form).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_state_invalid() {
    async fn handler(_: TypedMultipart<Data>) {
        panic!("should not be called");
    }
    let form = Form::new()
        .text("name", "data")
        .text("items", "bread")
        .text("items", "cheese")
        .text("default_value", "default")
        .text("position", "5");
    let res = create_test_client(handler, 10u32).post("/").multipart(form).await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(res.text().await, "position must be greater than or equal to 10");
}
