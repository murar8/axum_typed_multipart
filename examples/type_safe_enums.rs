use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromField, TryFromMultipart, TypedMultipart};

#[derive(Debug, TryFromField)]
pub enum Sex {
    Male,
    Female,
}

#[derive(TryFromMultipart)]
pub struct MultipartData {
    pub name: String,
    pub sex: Sex,
}

async fn test_multipart(multipart: TypedMultipart<MultipartData>) -> StatusCode {
    println!("name = {}, sex = {:?}", multipart.name, multipart.sex);
    StatusCode::OK
}

pub fn app() -> Router {
    Router::new().route("/", post(test_multipart))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}
