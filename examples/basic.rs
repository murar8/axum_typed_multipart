use axum::Router;
use axum::http::StatusCode;
use axum::routing::post;
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
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("0".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let addr = listener.local_addr()?;
    println!("Listening on http://{addr}");
    axum::serve(listener, app()).await
}
