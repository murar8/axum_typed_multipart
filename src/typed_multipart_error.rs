use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Error type for the [TryFromMultipart](crate::TryFromMultipart) trait.
#[derive(thiserror::Error, Debug)]
pub enum TypedMultipartError {
    #[error("request is malformed ({})", .source.body_text())]
    InvalidRequest {
        #[from]
        source: MultipartRejection,
    },

    #[error("request body is malformed ({})", .source.body_text())]
    InvalidRequestBody {
        #[from]
        source: MultipartError,
    },

    #[error("field '{field_name}' is required")]
    MissingField { field_name: String },

    #[error("field '{field_name}' must be of type '{wanted_type}'")]
    WrongFieldType { field_name: String, wanted_type: String },

    #[error("field '{field_name}' is already present")]
    DuplicateField { field_name: String },

    #[error("field '{field_name}' is not expected")]
    UnknownField { field_name: String },

    #[error("field '{field_name}' is larger than {limit_bytes} bytes")]
    FieldTooLarge { field_name: String, limit_bytes: usize },

    #[error(transparent)]
    Other {
        #[from]
        source: anyhow::Error,
    },
}

impl TypedMultipartError {
    fn get_status(&self) -> StatusCode {
        match self {
            | Self::MissingField { .. }
            | Self::WrongFieldType { .. }
            | Self::DuplicateField { .. }
            | Self::UnknownField { .. } => StatusCode::BAD_REQUEST,
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
mod tests {
    use super::*;
    use axum::body::HttpBody;
    use axum::extract::{FromRequest, Multipart};
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use axum::{async_trait, BoxError, Router};
    use axum_test_helper::TestClient;
    use bytes::Bytes;
    use reqwest::header;

    struct Data();

    #[async_trait]
    impl<S, B> FromRequest<S, B> for Data
    where
        B: HttpBody + Send + 'static,
        B::Data: Into<Bytes>,
        B::Error: Into<BoxError>,
        S: Send + Sync,
    {
        type Rejection = TypedMultipartError;

        async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
            let multipart = &mut Multipart::from_request(req, state).await?;
            while multipart.next_field().await?.is_some() {}
            unreachable!()
        }
    }

    fn create_client() -> TestClient {
        let handler = |_: Data| async { panic!("should never be called") };
        TestClient::new(Router::new().route("/", post(handler)))
    }

    #[tokio::test]
    async fn test_invalid_request() {
        let res = create_client().post("/").json(&"{}").send().await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(res.text().await.contains("request is malformed"));
    }

    #[tokio::test]
    async fn test_invalid_request_body() {
        let res = create_client()
            .post("/")
            .header(header::CONTENT_TYPE, "multipart/form-data; boundary=BOUNDARY")
            .body("BOUNDARY\r\n\r\nBOUNDARY--\r\n")
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(res.text().await.contains("request body is malformed"));
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
        let error = TypedMultipartError::WrongFieldType { field_name, wanted_type };
        assert_eq!(error.get_status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.to_string(), "field 'data' must be of type 'bar'");
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
