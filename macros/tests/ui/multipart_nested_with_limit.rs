use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
struct Inner {
    name: String,
}

#[derive(TryFromMultipart)]
struct Data {
    #[form_data(nested, limit = "1MB")]
    inner: Inner,
}

fn main() {}
