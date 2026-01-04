use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
struct Data {
    #[form_data(nested)]
    inner: &'static str,
}

fn main() {}
