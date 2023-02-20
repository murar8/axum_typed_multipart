use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum TypedMultipartError {
    #[error("request body is malformed")]
    UnparseableBody {
        #[from]
        source: MultipartRejection,
    },

    // TODO: add `field_name` in the error data.
    #[error("field is malformed")]
    UnparseableField {
        #[from]
        source: MultipartError,
    },

    #[error("field '{field_name}' must be of type '{field_type}'")]
    InvalidFieldType { field_name: String, field_type: String },

    #[error("field '{field_name}' is required")]
    MissingField { field_name: String },
}

impl TypedMultipartError {
    fn get_status(&self) -> StatusCode {
        match self {
            Self::InvalidFieldType { .. } | Self::MissingField { .. } => StatusCode::BAD_REQUEST,
            Self::UnparseableField { .. } | Self::UnparseableBody { .. } => {
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
