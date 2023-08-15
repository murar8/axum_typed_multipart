use std::net::SocketAddr;

use axum::{routing::post, Router};
use axum_typed_multipart::{TryFromField, TryFromMultipart, TypedMultipart};
use reqwest::StatusCode;

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

async fn test_multipart(TypedMultipart(multipart): TypedMultipart<MultipartData>) -> StatusCode {
    println!("name = {}, sex = {:?}", multipart.name, multipart.sex);
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(Router::new().route("/", post(test_multipart)).into_make_service())
        .await
        .unwrap();
}
