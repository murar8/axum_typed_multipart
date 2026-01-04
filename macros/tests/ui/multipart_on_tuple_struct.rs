use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
struct Data(String);

fn main() {}
