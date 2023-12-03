use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};

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
    let app = Router::new().route("/users/create", post(create_user)).into_make_service();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
