use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Error type for multipart parsing operations.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum TypedMultipartError {
    /// The request cannot be extracted as multipart form data.
    #[error("request is malformed ({})", .source.body_text())]
    InvalidRequest {
        #[from]
        /// The multipart extraction failure.
        source: MultipartRejection,
    },

    /// Reading or parsing the multipart body failed.
    #[error("request body is malformed ({})", .source.body_text())]
    InvalidRequestBody {
        #[from]
        /// The multipart body parsing failure.
        source: MultipartError,
    },

    /// A required field was not supplied.
    #[error("field '{field_name}' is required")]
    MissingField {
        /// The required field name.
        field_name: String,
    },

    /// A field could not be converted to its expected type.
    #[error("field '{field_name}' must be of type '{wanted_type}': {source}")]
    WrongFieldType {
        /// The field that failed conversion.
        field_name: String,
        /// The expected type name.
        wanted_type: String,
        /// The underlying conversion failure.
        source: anyhow::Error,
    },

    /// A non-repeated field was supplied more than once.
    #[error("field '{field_name}' is already present")]
    DuplicateField {
        /// The repeated field name.
        field_name: String,
    },

    /// A field was rejected because it was not expected.
    #[error("field '{field_name}' is not expected")]
    UnknownField {
        /// The unexpected field name.
        field_name: String,
    },

    /// A field value did not match an accepted enum variant.
    #[error("'{value}' is not a valid value for field '{field_name}'")]
    InvalidEnumValue {
        /// The field containing the invalid value.
        field_name: String,
        /// The rejected value.
        value: String,
    },

    /// A multipart field had no name.
    #[error("field name is empty")]
    NamelessField,

    /// A field exceeded its configured size limit.
    #[error("field '{field_name}' is larger than {limit_bytes} bytes")]
    FieldTooLarge {
        /// The oversized field name.
        field_name: String,
        /// The maximum permitted field size in bytes.
        limit_bytes: usize,
    },

    /// An application-specific or otherwise unclassified parsing failure.
    #[error(transparent)]
    Other {
        #[from]
        /// The underlying failure.
        source: anyhow::Error,
    },
}

impl TypedMultipartError {
    /// Returns the HTTP status used when converting this error into a response.
    #[must_use]
    pub fn get_status(&self) -> StatusCode {
        match self {
            | Self::MissingField { .. }
            | Self::WrongFieldType { .. }
            | Self::DuplicateField { .. }
            | Self::UnknownField { .. }
            | Self::InvalidEnumValue { .. }
            | Self::NamelessField { .. } => StatusCode::BAD_REQUEST,
            | Self::FieldTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            | Self::InvalidRequest { source } => source.status(),
            | Self::InvalidRequestBody { source } => source.status(),
            | Self::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for TypedMultipartError {
    fn into_response(self) -> Response {
        (self.get_status(), self.to_string()).into_response()
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use axum::Router;
    use axum::extract::{FromRequest, Multipart, Request};
    use axum::http::{StatusCode, header};
    use axum::routing::post;
    use axum_test_helper::TestClient;

    struct Data();

    impl<S> FromRequest<S> for Data
    where
        S: Send + Sync,
    {
        type Rejection = TypedMultipartError;

        async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
            let multipart = &mut Multipart::from_request(req, state).await?;
            while multipart.next_field().await?.is_some() {}
            unreachable!()
        }
    }

    fn create_client() -> TestClient {
        async fn handler(_: Data) {
            panic!("should never be called")
        }
        TestClient::new(Router::new().route("/", post(handler)))
    }

    #[tokio::test]
    async fn test_invalid_request() {
        let res = create_client().post("/").json(&"{}").send().await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(res.text().await.unwrap().contains("request is malformed"));
    }

    #[tokio::test]
    async fn test_invalid_request_body() {
        let res = create_client()
            .post("/")
            .header(header::CONTENT_TYPE.as_str(), "multipart/form-data; boundary=BOUNDARY")
            .body("BOUNDARY\r\n\r\nBOUNDARY--\r\n")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(res.text().await.unwrap().contains("request body is malformed"));
    }

    #[tokio::test]
    async fn test_missing_field() {
        let field_name = "data".to_string();
        let error = TypedMultipartError::MissingField { field_name };
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "field 'data' is required");
    }

    #[tokio::test]
    async fn test_wrong_field_type() {
        let field_name = "data".to_string();
        let wanted_type = "bar".to_string();
        let source = anyhow::anyhow!("invalid type");
        let error = TypedMultipartError::WrongFieldType { field_name, wanted_type, source };
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "field 'data' must be of type 'bar': invalid type");
    }

    #[tokio::test]
    async fn test_duplicate_field() {
        let field_name = "data".to_string();
        let error = TypedMultipartError::DuplicateField { field_name };
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "field 'data' is already present");
    }

    #[tokio::test]
    async fn test_unknown_field() {
        let field_name = "data".to_string();
        let error = TypedMultipartError::UnknownField { field_name };
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "field 'data' is not expected");
    }

    #[tokio::test]
    async fn test_invalid_enum_value() {
        let field_name = "status".to_string();
        let value = "invalid".to_string();
        let error = TypedMultipartError::InvalidEnumValue { field_name, value };
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "'invalid' is not a valid value for field 'status'");
    }

    #[tokio::test]
    async fn test_nameless_field() {
        let error = TypedMultipartError::NamelessField;
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "field name is empty");
    }

    #[tokio::test]
    async fn test_field_too_large() {
        let field_name = "data".to_string();
        let limit_bytes = 42;
        let error = TypedMultipartError::FieldTooLarge { field_name, limit_bytes };
        assert_eq!(error.get_status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(error.to_string(), "field 'data' is larger than 42 bytes");
    }

    #[tokio::test]
    async fn test_other() {
        let source = anyhow::anyhow!("data");
        let error = TypedMultipartError::Other { source };
        assert_eq!(error.get_status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.to_string(), "data");
    }
}
