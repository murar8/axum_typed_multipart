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
///     #[form_data(limit = "1MiB")]
///     file: TempFile,
/// }
/// ```
#[derive(Debug)]
pub struct TempFile(NamedTempFile);

impl TempFile {
    /// Persist the data permanently at the supplied `path`.
    ///
    /// When `replace` is `true` the file at the target path will be replaced if
    /// it exists.
    pub fn persist<P: AsRef<Path>>(self, path: P, replace: bool) -> Result<File, PersistError> {
        match replace {
            true => self.0.persist(path),
            false => self.0.persist_noclobber(path),
        }
    }
}

#[async_trait]
impl TryFromField for TempFile {
    async fn try_from_field(
        mut field: Field<'_>,
        limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let temp_file = NamedTempFile::new().map_err(anyhow::Error::new)?;
        let std_file = temp_file.reopen().map_err(anyhow::Error::new)?;
        let mut async_file = AsyncFile::from_std(std_file);
        let mut size_bytes = 0;

        while let Some(chunk) = field.chunk().await? {
            if let Some(limit_bytes) = limit_bytes {
                if size_bytes + chunk.len() > limit_bytes {
                    let field_name = field.name().unwrap().to_string();
                    return Err(TypedMultipartError::FieldTooLarge { field_name, limit_bytes });
                }
            }
            async_file.write_all(&chunk).await.map_err(anyhow::Error::new)?;
            size_bytes += chunk.len();
        }

        async_file.flush().await.map_err(anyhow::Error::new)?;

        Ok(TempFile(temp_file))
    }
}
