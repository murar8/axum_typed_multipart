use axum::http::StatusCode;
use axum::routing::post;
use axum::Router;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};

#[derive(TryFromMultipart)]
struct Address {
    street: String,
    city: String,
}

#[derive(TryFromMultipart)]
struct Person {
    name: String,
    #[form_data(nested)]
    address: Address,
}

#[derive(TryFromMultipart)]
struct CreateTeamRequest {
    team_name: String,
    #[form_data(nested)]
    manager: Option<Person>, // Optional nested struct
    #[form_data(nested)]
    members: Vec<Person>, // Array of nested structs
}

// Expected form fields:
// - team_name
// - manager.name, manager.address.street, manager.address.city (optional)
// - members[0].name, members[0].address.street, members[0].address.city
// - members[1].name, members[1].address.street, members[1].address.city

fn print_team(data: &CreateTeamRequest) {
    println!("Team: {}", data.team_name);
    for member in &data.members {
        println!("Member: {} ({}, {})", member.name, member.address.street, member.address.city);
    }
    if let Some(ref manager) = data.manager {
        println!(
            "Manager: {} ({}, {})",
            manager.name, manager.address.street, manager.address.city
        );
    }
}

async fn create_team(data: TypedMultipart<CreateTeamRequest>) -> StatusCode {
    print_team(&data);
    StatusCode::CREATED
}

pub fn app() -> Router {
    Router::new().route("/teams", post(create_team))
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or("0".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}
