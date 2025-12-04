use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};

#[derive(TryFromMultipart)]
struct Address {
    city: String,
    country: String,
}

#[derive(TryFromMultipart)]
struct CreateUserRequest {
    name: String,
    #[form_data(flatten)]
    address: Address,
}

async fn create_user(data: TypedMultipart<CreateUserRequest>) -> StatusCode {
    // Form fields: name, address.city, address.country
    println!("{} from {}, {}", data.name, data.address.city, data.address.country);
    StatusCode::CREATED
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/users/create", post(create_user)).into_make_service();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
