use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use std::net::SocketAddr;
use std::path::Path;
use tempfile::NamedTempFile;

#[derive(TryFromMultipart)]
struct UploadAssetRequest {
    // This field will be limited to the total size of the request body.
    #[form_data(limit = "unlimited")]
    image: FieldData<NamedTempFile>,

    // This field will be limited to the default size of 1MiB.
    author: String,
}

async fn upload_asset(
    TypedMultipart(UploadAssetRequest { image, author }): TypedMultipart<UploadAssetRequest>,
) -> StatusCode {
    let file_name = image.metadata.file_name.unwrap_or(String::from("data.bin"));
    let path = Path::new("/tmp").join(author).join(file_name);

    match image.contents.persist(path) {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", post(upload_asset))
        // The default axum body size limit is 2MiB, so we increase it to 1GiB.
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));

    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(router.into_make_service())
        .await
        .unwrap();
}
