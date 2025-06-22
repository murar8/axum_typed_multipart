use crate::TypedMultipartError;
use axum::extract::multipart::Field;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::{StreamExt as _, TryStreamExt as _};
use std::mem;

/// Converts a string literal representation of truth to true or false.
///
/// Adapted from https://docs.rs/crate/clap_builder/4.5.40/source/src/util/str_to_bool.rs
pub fn str_to_bool(val: impl AsRef<str>) -> Option<bool> {
    const TRUE_LITERALS: [&str; 6] = ["y", "yes", "t", "true", "on", "1"];
    const FALSE_LITERALS: [&str; 6] = ["n", "no", "f", "false", "off", "0"];

    let pat: &str = &val.as_ref().to_lowercase();
    if TRUE_LITERALS.contains(&pat) {
        Some(true)
    } else if FALSE_LITERALS.contains(&pat) {
        Some(false)
    } else {
        None
    }
}

pub fn get_chunks<'a>(
    field: Field<'a>,
    limit_bytes: Option<usize>,
    field_name: &'a mut String,
) -> impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin + 'a {
    let mut size_bytes = 0;
    field.map_err(TypedMultipartError::from).map(move |chunk| {
        if let Ok(chunk) = chunk.as_ref() {
            size_bytes += chunk.len();

            if let Some(limit_bytes) = limit_bytes {
                if size_bytes > limit_bytes {
                    return Err(TypedMultipartError::FieldTooLarge {
                        field_name: mem::take(field_name),
                        limit_bytes,
                    });
                }
            }
        }

        chunk
    })
}
