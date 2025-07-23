use axum::extract::multipart::Field;
use axum_typed_multipart::{FieldMetadata, TypedMultipart, TypedMultipartError};
use axum_typed_multipart_macros::TryFromMultipart;
use futures_util::TryStreamExt;
use tempfile_3::NamedTempFile;
use tokio::io::AsyncWriteExt;

struct NamedTempFileAtPath(NamedTempFile);

#[derive(Clone)]
struct State {
    path: std::path::PathBuf,
}

#[async_trait::async_trait]
impl axum_typed_multipart::TryFromFieldWithState<State> for NamedTempFileAtPath {
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        state: &State,
    ) -> Result<Self, TypedMultipartError> {
        let metadata = FieldMetadata::from(&field);
        let field_name = metadata.name.clone().unwrap_or(String::new());
        let mut size_bytes = 0;

        // We can't use the TryFromField impl for NamedTempFile as we need to inject the path at construction
        // There is no TryFromChunksWithState trait which we can use to get a TryFromFieldWithState impl so we need to
        // implement the size checking logic ourselves

        let temp_file = tempfile_3::Builder::new()
            .tempfile_in(&state.path)
            .map_err(|err| TypedMultipartError::Other { source: anyhow::Error::new(err) })?;
        let std_file = temp_file
            .reopen()
            .map_err(|err| TypedMultipartError::Other { source: anyhow::Error::new(err) })?;
        let mut async_file = tokio::fs::File::from_std(std_file);

        // actually write the stream to the file
        let _ = field
            .map_err(TypedMultipartError::from)
            .try_fold(
                (&mut async_file, &mut size_bytes, &limit_bytes, &field_name),
                async |(async_file, size_bytes, limit_bytes, field_name), chunk| {
                    // calculate the size after we've written this chunk to the buffer and check if that would exceed the limit
                    let new_size_bytes = *size_bytes + chunk.len();
                    if let Some(limit_bytes) = limit_bytes {
                        if new_size_bytes > *limit_bytes {
                            return Err(TypedMultipartError::FieldTooLarge {
                                field_name: field_name.clone(),
                                limit_bytes: limit_bytes.clone(),
                            });
                        }
                    }

                    async_file.write_all(&chunk).await.map_err(|err| {
                        TypedMultipartError::Other { source: anyhow::Error::new(err) }
                    })?;

                    *size_bytes = new_size_bytes;

                    Ok((async_file, size_bytes, limit_bytes, field_name))
                },
            )
            .await?;

        async_file
            .flush()
            .await
            .map_err(|err| TypedMultipartError::Other { source: anyhow::Error::new(err) })?;

        Ok(NamedTempFileAtPath(temp_file))
    }
}

#[derive(TryFromMultipart)]
#[try_from_multipart(state = State)]
struct UploadFileRequest {
    file: NamedTempFileAtPath,
}

async fn update_position(
    TypedMultipart(data): TypedMultipart<UploadFileRequest>,
) -> axum::http::StatusCode {
    println!("file was uploaded to path '{:?}'", data.file.0.path());
    axum::http::StatusCode::OK
}

#[tokio::main]
async fn main() {
    let state = State { path: std::path::PathBuf::from("./uploads") };
    tokio::fs::create_dir_all(&state.path).await.unwrap();
    let app = axum::Router::new()
        .route("/file/upload", axum::routing::post(update_position))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:10000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
