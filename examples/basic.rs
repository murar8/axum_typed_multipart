use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use std::net::SocketAddr;

#[derive(TryFromMultipart)]
struct RequestData {
    first_name: String,
    last_name: String,
}

async fn handler(
    TypedMultipart(RequestData { first_name, last_name }): TypedMultipart<RequestData>,
) -> String {
    format!("{} {}", first_name, last_name)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(handler));
    let addres = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addres).serve(app.into_make_service()).await.unwrap();
}
