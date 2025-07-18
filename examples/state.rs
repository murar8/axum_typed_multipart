use async_trait::async_trait;
use axum::extract::multipart::Field;
use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{
    TryFromField as _, TryFromFieldWithState, TryFromMultipart, TypedMultipart, TypedMultipartError,
};

#[derive(Clone)]
struct AppState {
    min_position: u32,
}

struct Position(u32);

#[async_trait]
impl TryFromFieldWithState<AppState> for Position {
    async fn try_from_field_with_state(
        field: Field<'_>,
        limit_bytes: Option<usize>,
        state: &AppState,
    ) -> Result<Self, TypedMultipartError> {
        let position = u32::try_from_field(field, limit_bytes).await?;
        if position >= state.min_position {
            Ok(Position(position))
        } else {
            Err(TypedMultipartError::Other {
                source: anyhow::anyhow!(
                    "position must be greater than or equal to {}",
                    state.min_position
                ),
            })
        }
    }
}

#[derive(TryFromMultipart)]
#[try_from_multipart(state = AppState)]
struct UpdatePositionRequest {
    name: String,
    position: Position,
}

async fn update_position(
    TypedMultipart(data): TypedMultipart<UpdatePositionRequest>,
) -> StatusCode {
    println!("name = '{}'", data.name);
    println!("position = '{}'", data.position.0);
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/position/update", post(update_position))
        .with_state(AppState { min_position: 10 });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
