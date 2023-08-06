use crate::{TryFromField, TypedMultipartError};
use axum::async_trait;
use axum::extract::multipart::Field;
use std::fs::File;
use std::path::Path;
use tempfile::{NamedTempFile, PersistError};
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;

/// Stream the field data on the file system using a temporary file.
///
/// Once the data is saved you must save it permanently to the file system using
/// the `persist` method.
///
/// This is especially useful for large file uploads where you might not be able
/// to fit all the file contents in RAM.
///
/// If the program exits before the destructor is run, the temporary file will
/// not be deleted. For more details about this check the [NamedTempFile]
/// documentation.
///
/// ## Example
/// ```rust
/// use axum_typed_multipart::{TempFile, TryFromMultipart, TypedMultipart};
///
/// #[derive(TryFromMultipart)]
/// struct FileUpload {
///     file: TempFile,
/// }
/// ```
pub struct TempFile(NamedTempFile);

impl TempFile {
    /// Persist the data permanently at the supplied `path`.
    ///
    /// When `replace` is `true` the file at the target path will be replaced if
    /// it exists.
    pub async fn persist<P: AsRef<Path>>(
        self,
        path: P,
        replace: bool,
    ) -> Result<File, PersistError> {
        match replace {
            true => self.0.persist(path),
            false => self.0.persist_noclobber(path),
        }
    }
}

#[async_trait]
impl TryFromField for TempFile {
    async fn try_from_field(mut field: Field<'_>) -> Result<Self, TypedMultipartError> {
        let temp_file = NamedTempFile::new().map_err(anyhow::Error::new)?;
        let std_file = temp_file.reopen().map_err(anyhow::Error::new)?;
        let mut async_file = AsyncFile::from_std(std_file);

        while let Some(chunk) = field.chunk().await? {
            async_file.write_all(&chunk).await.map_err(anyhow::Error::new)?;
        }

        async_file.flush().await.map_err(anyhow::Error::new)?;

        Ok(TempFile(temp_file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Multipart;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::{Form, Part};
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_temp_file() {
        async fn handler(mut multipart: Multipart) {
            let field = multipart.next_field().await.unwrap().unwrap();
            let file = TempFile::try_from_field(field).await.unwrap();
            let path = TempDir::new().unwrap().into_path().join("potato.txt");
            file.persist(&path, true).unwrap();
            let contents = fs::read_to_string(path).unwrap();
            assert_eq!(contents, "test");
        }

        let part = Part::text("test").file_name("test.txt").mime_str("text/plain").unwrap();

        TestClient::new(Router::new().route("/", post(handler)))
            .post("/")
            .multipart(Form::new().part("input_file", part))
            .send()
            .await;
    }
}
