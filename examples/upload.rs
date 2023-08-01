use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{FieldData, TempFile, TryFromMultipart, TypedMultipart};
use std::net::SocketAddr;
use tempfile::TempDir;

#[derive(TryFromMultipart)]
struct RequestData {
    // We can make this field unlimited because the request body is limited to
    // 1GiB.
    #[form_data(limit = "unlimited")]
    file: FieldData<TempFile>,

    // This field will be limited to the default size of 1MiB.
    author: String,
}

async fn handler(
    TypedMultipart(RequestData { file, author }): TypedMultipart<RequestData>,
) -> String {
    let file_name = file.metadata.file_name.unwrap_or(String::from("data.bin"));
    let dir = TempDir::new().unwrap();
    let path = dir.path().join(format!("{}-{}", author, file_name));
    file.contents.persist(&path, true).unwrap();
    path.to_str().unwrap().to_string()
}

#[tokio::main]
async fn main() {
    let req_size_limit_bytes = 1024 * 1024 * 1024; // 1GiB

    // The default limit is 1MiB, so we need to increase it to 1GiB.
    let app =
        Router::new().route("/", post(handler)).layer(DefaultBodyLimit::max(req_size_limit_bytes));

    let addres = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addres).serve(app.into_make_service()).await.unwrap();
}
