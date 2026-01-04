use axum_typed_multipart::TryFromField;

#[derive(TryFromField)]
#[try_from_field(rename_all = "invalid_case")]
enum Data {
    Variant,
}

fn main() {}
