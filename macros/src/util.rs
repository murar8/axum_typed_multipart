/// Check if the supplied type matches the [Option] signature and return the
/// enclosed type if it does.
///
/// Note that this method is not guaranteed to work on every possible input
/// since we don't have access to type information in the AST representation.
///
/// Adapted from https://stackoverflow.com/a/56264023
pub fn get_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    let path = match ty {
        syn::Type::Path(type_path) if type_path.qself.is_none() => &type_path.path,
        _ => return None,
    };

    let full_path =
        path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");

    match full_path.as_ref() {
        "Option" | "std::option::Option" | "core::option::Option" => {}
        _ => return None,
    }

    let argument = match path.segments.last().unwrap().arguments {
        syn::PathArguments::AngleBracketed(ref params) => params.args.first(),
        _ => return None,
    };

    match argument {
        Some(syn::GenericArgument::Type(ref ty)) => Some(ty),
        _ => None,
    }
}
