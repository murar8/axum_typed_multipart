use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::post;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use tempfile_3::NamedTempFile;

#[derive(TryFromMultipart)]
struct UploadAssetRequest {
    // Field size limits are disabled by default. The `limit` parameter can be used
    // to set a specific size limit in bytes, like '5MiB' or '1GiB'. The value
    // "unlimited" explicitly disables the limit (same as the default behavior).
    #[form_data(limit = "unlimited")]
    image: FieldData<NamedTempFile>,

    // This field has no size limit since limits are disabled by default.
    author: String,
}

async fn upload_asset(
    TypedMultipart(UploadAssetRequest { image, author }): TypedMultipart<UploadAssetRequest>,
) -> StatusCode {
    let Ok(dir) = tempfile_3::tempdir() else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let file_name = image.metadata.file_name.unwrap_or(String::from("data.bin"));
    let path = dir.path().join(format!("{author}_{file_name}"));

    match image.contents.persist(path) {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn app() -> Router {
    Router::new()
        .route("/", post(upload_asset))
        // The default axum body size limit is 2MiB, so we increase it to 1GiB.
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("0".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let addr = listener.local_addr()?;
    println!("Listening on http://{addr}");
    axum::serve(listener, app()).await
}
