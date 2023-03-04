use axum::extract::multipart::Field;
use axum::http::HeaderMap;

/// Additional information about the file supplied by the client in the request.
pub struct FieldMetadata {
    /// Name of the HTML field in the form.
    ///
    /// If the [TryFromMultipart](crate::TryFromMultipart)
    /// implementation for the struct where this field is used was generated
    /// using the derive macro it will make it safe to unwrap this value since
    /// the field name must always be present to allow for mapping it to a
    /// struct field.
    ///
    /// Extracted from the
    /// [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition)
    /// header.
    pub name: Option<String>,

    /// Original name of the file transmitted.
    ///
    /// The filename is always optional and must not be used blindly by the
    /// application: path information should be stripped, and conversion to the
    /// server file system rules should be done.
    ///
    /// Extracted from the
    /// [`Content-Disposition`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Disposition)
    /// header.
    pub file_name: Option<String>,

    /// MIME type of the field.
    ///
    /// Extracted from the
    /// [`Content-Type`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type)
    /// header.
    pub content_type: Option<String>,

    /// HTTP headers sent with the field.
    pub headers: HeaderMap,
}

impl From<&Field<'_>> for FieldMetadata {
    fn from(field: &Field) -> Self {
        Self {
            name: field.name().map(String::from),
            file_name: field.file_name().map(String::from),
            content_type: field.content_type().map(String::from),
            headers: field.headers().clone(),
        }
    }
}
