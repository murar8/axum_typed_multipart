use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use axum_typed_multipart::{BaseMultipart, TryFromMultipart, TypedMultipartError};
use serde::Serialize;
use std::net::SocketAddr;

// Step 1: Define a custom error type.
#[derive(Serialize)]
struct CustomError {
    message: String,
    status: u16,
}

// Step 2: Implement `IntoResponse` for the custom error type.
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        Json(self).into_response()
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
async fn update_position(
    CustomMultipart { data, .. }: CustomMultipart<UpdatePositionRequest>,
) -> StatusCode {
    println!("name = '{}'", data.name);
    println!("position = '{}'", data.position);
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(Router::new().route("/position/update", post(update_position)).into_make_service())
        .await
        .unwrap();
}
