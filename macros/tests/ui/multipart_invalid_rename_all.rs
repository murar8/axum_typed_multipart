use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
#[try_from_multipart(rename_all = "invalid_case")]
struct Data {
    field: String,
}

fn main() {}
