use axum_typed_multipart::TryFromField;

#[derive(TryFromField)]
struct Data {
    field: String,
}

fn main() {}
