use crate::{TryFromField, TypedMultipartError};
use axum::async_trait;
use axum::extract::multipart::Field;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::{NamedTempFile, PersistError};

/// Stream the field data on the file system using a temporary file.
///
/// Once the data is saved you must save it permanently to the file system using
/// the `persist` method.
///
/// This is especially useful for large file uploads where you might not be able
/// to store all the file contents into memory.
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
        let mut file = NamedTempFile::new().map_err(anyhow::Error::new)?;

        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk).map_err(anyhow::Error::new)?;
        }

        Ok(TempFile(file))
    }
}
