/// Check if the supplied type matches at least one of the provided signatures.
///
/// Note that this method is not guaranteed to work on every possible input
/// since we don't have access to type information in the AST representation.
///
/// Adapted from https://stackoverflow.com/a/56264023
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn matches_signature(ty: &syn::Type, signatures: &[&str]) -> bool {
    let path = match ty {
        syn::Type::Path(type_path) if type_path.qself.is_none() => &type_path.path,
        _ => return false,
    };

    let signature =
        path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");

    signatures.contains(&signature.as_ref())
}

/// Strips leading r# from the ident. Used to convert idents to string literals.
pub fn strip_leading_rawlit(s: &str) -> String {
    if s.starts_with("r#") {
        s.chars().skip(2).collect()
    } else {
        s.to_owned()
    }
}

/// Check if the supplied type matches the [Option] signature.
pub fn matches_option_signature(ty: &syn::Type) -> bool {
    matches_signature(ty, &["Option", "std::option::Option", "core::option::Option"])
}

/// Check if the supplied type matches the [Vec] signature.
pub fn matches_vec_signature(ty: &syn::Type) -> bool {
    matches_signature(ty, &["Vec", "std::vec::Vec"])
}
