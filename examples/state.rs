use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TypedMultipart, TypedMultipartError};
use axum_typed_multipart_macros::TryFromMultipart;

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
            // SECURITY: When implementing TryFromFieldWithState, you must manually handle size limits.
            // The stateful variant does not have a TryFromChunksWithState trait, so automatic
            // size checking is not available.
            //
            // If you are not using the #[form_data(limit = "...")] attribute, then limit_bytes
            // will be None and you can safely ignore size checking.
            //
            // However, if you are using #[form_data(limit = "...")], you MUST check the limit_bytes
            // parameter progressively as shown below to enforce the specified limit and prevent
            // denial-of-service attacks from malicious clients sending unbounded data.
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

#[tokio::main]
async fn main() {
    let allowed_roles = ["admin", "editor", "viewer", "guest"];
    let state =
        State { allowed_values: allowed_roles.into_iter().map(ToString::to_string).collect() };
    let app = Router::new().route("/user/update", post(update_user)).with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
