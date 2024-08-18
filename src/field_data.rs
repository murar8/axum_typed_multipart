use crate::{TryFromField, TypedMultipartError};
use axum::async_trait;
use axum::extract::multipart::Field;
use axum::http::HeaderMap;

/// Additional information about the file supplied by the client in the request.
#[derive(Debug, Clone, Default)]
pub struct FieldMetadata {
    /// Name of the HTML field in the form.
    ///
    /// If the [TryFromMultipart](crate::TryFromMultipart) implementation for
    /// the struct where this field is used was generated using the derive macro
    /// it will make it safe to unwrap this value since the field name must
    /// always be present to allow for mapping it to a struct field.
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

/// Wrapper struct that allows to retrieve both the field contents and the
/// additional metadata provided by the client.
///
/// This is mainly useful for file uploads but can be used for every field where
/// you need access to the metadata.
///
/// If the generic argument implements [TryFromField](crate::TryFromField) the
/// struct will implement the trait itself.
///
/// ## Example
///
/// ```rust
/// use axum_typed_multipart::{FieldData, TryFromMultipart};
///
/// #[derive(TryFromMultipart)]
/// struct Data {
///     data: FieldData<String>,
/// }
/// ```
#[derive(Debug)]
pub struct FieldData<T> {
    pub metadata: FieldMetadata,
    pub contents: T,
}

#[async_trait]
impl<T: TryFromField> TryFromField for FieldData<T> {
    async fn try_from_field(
        field: Field<'_>,
        limit_bytes: Option<usize>,
    ) -> Result<Self, TypedMultipartError> {
        let metadata = FieldMetadata::from(&field);
        let contents = T::try_from_field(field, limit_bytes).await?;
        Ok(Self { metadata, contents })
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use crate::TryFromField;
    use axum::extract::Multipart;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum::Router;
    use axum_test_helper::TestClient;
    use reqwest::multipart::{Form, Part};

    #[tokio::test]
    async fn test_field_data() {
        let handler = |mut multipart: Multipart| async move {
            let field = multipart.next_field().await.unwrap().unwrap();
            let field_data = FieldData::<String>::try_from_field(field, None).await.unwrap();

            assert_eq!(field_data.metadata.name.unwrap(), "input_file");
            assert_eq!(field_data.metadata.file_name.unwrap(), "test.txt");
            assert_eq!(field_data.metadata.content_type.unwrap(), "text/plain");
            assert_eq!(field_data.contents, "test");
        };

        let part = Part::text("test").file_name("test.txt").mime_str("text/plain").unwrap();

        let res = TestClient::new(Router::new().route("/", post(handler)))
            .await
            .post("/")
            .multipart(Form::new().part("input_file", part))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);
    }
}
