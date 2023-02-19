use axum::extract::multipart::{MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum TypedMultipartError {
    InvalidBody(MultipartRejection),
    InvalidField(MultipartError),
    MissingField(String),
}

impl IntoResponse for TypedMultipartError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidBody(e) => e.into_response(),
            Self::InvalidField(e) => {
                (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
            }
            Self::MissingField(field) => {
                let message = format!("field '{}' is required", field);
                (StatusCode::BAD_REQUEST, message).into_response()
            }
        }
    }
}

impl From<MultipartRejection> for TypedMultipartError {
    fn from(error: MultipartRejection) -> Self {
        TypedMultipartError::InvalidBody(error)
    }
}

impl From<MultipartError> for TypedMultipartError {
    fn from(error: MultipartError) -> Self {
        TypedMultipartError::InvalidField(error)
    }
}
