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

pub fn app() -> Router {
    Router::new().route("/users/create", post(create_user))
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or("0".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}
