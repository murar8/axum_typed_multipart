use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
struct Data {
    #[form_data(limit = "invalid")]
    field: String,
}

fn main() {}
