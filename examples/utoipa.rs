use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_rapidoc::RapiDoc;

#[derive(OpenApi)]
#[openapi(paths(file_upload), components(schemas(FileUpload, Status)))]
struct ApiDoc;

#[derive(TryFromMultipart, ToSchema)]
pub struct FileUpload {
    /// User's name
    #[schema(example = "John Doe")]
    name: String,

    /// File or files to upload
    #[form_data(limit = "2MiB")]
    #[schema(value_type = Vec<u8>)]
    file: Vec<FieldData<Bytes>>,
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct Status {
    /// Status
    #[schema(example = "error")]
    pub status: String,
    /// What went wrong
    #[schema(example = "Could not open file")]
    pub error: Option<String>,
}

/// Upload a file
///
/// Accepts a user's name and a file
#[utoipa::path(
    post,
    path = "/upload",
    request_body(content_type = "multipart/form-data", content = FileUpload),
    responses(
        (status = 200, description = "File uploaded successfully", body = Status),
    ),
    tag = "Upload"
)]
async fn file_upload(
    TypedMultipart(FileUpload { name, file }): TypedMultipart<FileUpload>,
) -> Response {
    println!("User's name: {name}");
    for f in file.into_iter() {
        println!("Filename: {:?}", f.metadata.file_name);
    }
    (StatusCode::OK, Json(Status { status: "ok".into(), error: None })).into_response()
}

pub fn app() -> Router {
    Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/"))
        .route("/upload", post(file_upload))
}

#[tokio::main]
async fn main() {
    println!("Listening on http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}
