use axum_typed_multipart::TryFromField;

#[derive(TryFromField)]
enum Data {
    Variant(String),
}

fn main() {}
