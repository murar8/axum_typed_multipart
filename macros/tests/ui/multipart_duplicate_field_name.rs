use axum_typed_multipart::TryFromMultipart;

#[derive(TryFromMultipart)]
struct Data {
    #[form_data(field_name = "name")]
    first_name: String,
    #[form_data(field_name = "name")]
    last_name: String,
}

fn main() {}
