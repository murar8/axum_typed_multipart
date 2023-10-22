use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use std::net::SocketAddr;

#[derive(TryFromMultipart)]
struct CreateUserRequest {
    first_name: String,
    last_name: String,
}

async fn create_user(data: TypedMultipart<CreateUserRequest>) -> StatusCode {
    println!("name: '{} {}'", data.first_name, data.last_name); // Your logic here.
    StatusCode::CREATED
}

#[tokio::main]
async fn main() {
    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(Router::new().route("/users/create", post(create_user)).into_make_service())
        .await
        .unwrap();
}
