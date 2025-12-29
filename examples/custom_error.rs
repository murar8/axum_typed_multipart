use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use axum_typed_multipart::{BaseMultipart, TryFromMultipart, TypedMultipartError};
use serde::Serialize;

// Step 1: Define a custom error type.
#[derive(Serialize)]
struct CustomError {
    message: String,
    status: u16,
}

// Step 2: Implement `IntoResponse` for the custom error type.
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

// Step 3: Implement `From<TypedMultipartError>` for the custom error type.
impl From<TypedMultipartError> for CustomError {
    fn from(error: TypedMultipartError) -> Self {
        Self { message: error.to_string(), status: error.get_status().into() }
    }
}

// Step 4: Define a type alias for the multipart request (Optional).
type CustomMultipart<T> = BaseMultipart<T, CustomError>;

#[derive(TryFromMultipart)]
struct UpdatePositionRequest {
    name: String,
    position: u32,
}

// Step 5: Define a handler that takes the custom multipart as argument.
// If the request is malformed, a `CustomError` will be returned.
async fn update_position(data: CustomMultipart<UpdatePositionRequest>) -> StatusCode {
    println!("name = '{}'", data.name);
    println!("position = '{}'", data.position);
    StatusCode::OK
}

pub fn app() -> Router {
    Router::new().route("/position/update", post(update_position))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}
