use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart, TypedMultipartError};

/// A field that validates its value against application state.
#[derive(Debug)]
struct ValidatedField(String);

/// Application state containing validation rules.
#[derive(Clone)]
struct State {
    allowed_values: Vec<String>,
}

#[async_trait::async_trait]
impl axum_typed_multipart::TryFromFieldWithState<State> for ValidatedField {
    async fn try_from_field_with_state(
        mut field: axum::extract::multipart::Field<'_>,
        limit_bytes: Option<usize>,
        state: &State,
    ) -> Result<Self, TypedMultipartError> {
        let mut value = String::new();

        while let Some(chunk) = field.chunk().await.map_err(anyhow::Error::from)? {
            // SECURITY: Manual size limit handling is required for TryFromFieldWithState.
            // Unlike TryFromField which can leverage TryFromChunks for automatic size checking,
            // the stateful variant requires explicit implementation.
            //
            // When limit_bytes will be None:
            // - Your type is private (not exposed in public API) AND
            // - You don't use #[form_data(limit = "...")] on fields of this type
            // In this case, you control all usage and can skip size checking if appropriate.
            //
            // When limit_bytes may have a value:
            // - Your type is public (part of your API), OR
            // - You use #[form_data(limit = "...")] on any field of this type
            // You MUST enforce the limit to prevent denial-of-service attacks from unbounded uploads.
            if let Some(limit) = limit_bytes {
                if value.len() + chunk.len() > limit {
                    return Err(TypedMultipartError::FieldTooLarge {
                        field_name: field.name().unwrap_or("unknown").to_string(),
                        limit_bytes: limit,
                    });
                }
            }
            value.push_str(std::str::from_utf8(&chunk).map_err(anyhow::Error::from)?);
        }

        if state.allowed_values.contains(&value) {
            Ok(ValidatedField(value))
        } else {
            Err(TypedMultipartError::Other {
                source: anyhow::anyhow!("Value '{}' is not allowed", value),
            })
        }
    }
}

#[derive(TryFromMultipart)]
#[try_from_multipart(state = State)]
struct UpdateUserRequest {
    #[form_data(limit = "100B")]
    role: ValidatedField,
}

async fn update_user(TypedMultipart(data): TypedMultipart<UpdateUserRequest>) -> StatusCode {
    println!("User role updated to: '{}'", data.role.0);
    StatusCode::OK
}

pub fn app() -> Router {
    let allowed_roles = ["admin", "editor", "viewer", "guest"];
    let state =
        State { allowed_values: allowed_roles.into_iter().map(ToString::to_string).collect() };
    Router::new().route("/user/update", post(update_user)).with_state(state)
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or("0".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}
