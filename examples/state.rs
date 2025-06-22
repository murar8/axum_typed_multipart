use async_trait::async_trait;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{
    FieldMetadata, StatefulTryFromChunks, TryFromChunks as _, TryFromMultipart, TypedMultipart,
    TypedMultipartError,
};
use bytes::Bytes;
use futures_core::Stream;

#[derive(Clone)]
struct AppState {
    pub max_position: usize,
}

struct Position(pub usize);

#[async_trait]
impl StatefulTryFromChunks<AppState> for Position {
    async fn try_from_chunks_with_state(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
        state: &AppState,
    ) -> Result<Self, TypedMultipartError> {
        let field_name = metadata.name.clone().unwrap();
        let position = usize::try_from_chunks(chunks, metadata).await?;
        if position > state.max_position {
            Err(TypedMultipartError::FieldTooLarge { field_name, limit_bytes: state.max_position })
        } else {
            Ok(Position(position))
        }
    }
}

#[derive(TryFromMultipart)]
#[try_from_multipart(state_type = AppState)]
struct CreateUserRequest {
    first_name: String,
    last_name: String,
    position: Position,
}

async fn create_user(data: TypedMultipart<CreateUserRequest>) -> StatusCode {
    println!("first_name: {}", data.first_name);
    println!("last_name: {}", data.last_name);
    println!("position: {}", data.position.0);
    StatusCode::CREATED
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users/create", post(create_user))
        .with_state(AppState { max_position: 10 });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
