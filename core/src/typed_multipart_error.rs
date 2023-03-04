use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum TypedMultipartError {
    #[error("request is malformed")]
    InvalidRequest {
        #[from]
        source: MultipartRejection,
    },

    #[error("request body is malformed")]
    InvalidRequestBody {
        #[from]
        source: MultipartError,
    },

    #[error("field '{field_name}' is required")]
    MissingField { field_name: String },

    #[error("field '{field_name}' must be of type '{field_type}'")]
    WrongFieldType { field_name: String, field_type: String },

    #[error(transparent)]
    Other {
        #[from]
        source: anyhow::Error,
    },
}

impl TypedMultipartError {
    fn get_status(&self) -> StatusCode {
        match self {
            Self::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingField { .. } | Self::WrongFieldType { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidRequest { .. } | Self::InvalidRequestBody { .. } => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
        }
    }
}

impl IntoResponse for TypedMultipartError {
    fn into_response(self) -> Response {
        (self.get_status(), self.to_string()).into_response()
    }
}
