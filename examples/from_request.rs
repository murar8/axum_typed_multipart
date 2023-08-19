use axum::routing::post;
use axum::Router;
use axum::async_trait;
use axum::RequestExt;
use axum::extract::{FromRequest, Form, Json};
use axum::response::{IntoResponse, Response};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use axum::http;
use http::{Request, status::StatusCode};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, TryFromMultipart, Deserialize)]
pub struct LoginForm {
    username: String,
    #[allow(dead_code)]
    password: String,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for LoginForm
where
    B: Send + 'static + axum::body::HttpBody,
    S: Send,
    Json<LoginForm>: FromRequest<(), B>,
    Form<LoginForm>: FromRequest<(), B>,
    B::Data: Into<axum::body::Bytes>,
    B::Error: Into<axum::BoxError> + Send + std::error::Error,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers();

        if let Some(mime) = headers.get(http::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()) {
            if mime.starts_with("application/json") {
                let Json(login_form): Json<LoginForm> = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(login_form);
            }

            if mime.starts_with("application/x-www-form-urlencoded") {
                let Form(login_form) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(login_form);
            }

            if mime.starts_with("multipart/form-data") {
                let TypedMultipart(login_form): TypedMultipart<LoginForm> = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(login_form);
            }
            Err(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response())
        } else {
            Err("No POST Content-Type".into_response())
        }
    }
}

async fn login(TypedMultipart(login_form): TypedMultipart<LoginForm>) -> Response {
    let msg = format!("username = {}", login_form.username);
    println!("{}", msg);
    (StatusCode::OK, msg).into_response()
}

#[tokio::main]
async fn main() {
    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(Router::new().route("/login", post(login)).into_make_service())
        .await
        .unwrap();
}
