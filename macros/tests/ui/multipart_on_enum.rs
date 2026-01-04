use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
enum Data {
    Variant,
}

fn main() {}
