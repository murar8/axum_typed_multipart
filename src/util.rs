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
