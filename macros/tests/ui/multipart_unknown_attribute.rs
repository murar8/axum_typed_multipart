use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
#[try_from_multipart(unknown_attr)]
struct Data {
    field: String,
}

fn main() {}
