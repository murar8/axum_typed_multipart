use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

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
