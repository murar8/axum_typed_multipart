/// Result of parsing a Vec suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VecIndex {
    /// Explicit numeric index like `[0]` or `[1]`
    Explicit(usize),
    /// Auto-append: `[]` or bare field name
    Auto,
}

/// Parses a Vec index from the start of the suffix.
///
/// Supports multiple formats:
/// - `[n]` - explicit numeric index (e.g., `[0]`, `[1]`)
/// - `[]` - auto-append (empty brackets)
/// - Empty or starts with `.` - auto-append (bare field name or nested field)
///
/// Returns:
/// - `Ok((VecIndex, rest))` on successful parse
/// - `Err(...)` if index format is invalid
pub(crate) fn parse_vec_index(suffix: &str) -> Result<(VecIndex, &str), anyhow::Error> {
    // Empty suffix or starts with dot: auto-append, pass suffix through
    if suffix.is_empty() || suffix.starts_with('.') {
        return Ok((VecIndex::Auto, suffix));
    }

    // Must start with '[' for bracket notation
    let rest = suffix
        .strip_prefix('[')
        .ok_or_else(|| anyhow::anyhow!("expected '[index]', '[]', or '.' but found '{suffix}'"))?;

    let end = rest.find(']').ok_or_else(|| anyhow::anyhow!("missing closing bracket"))?;
    let idx_str = &rest[..end];
    let rest = &rest[end + 1..];

    // Empty brackets: auto-append
    if idx_str.is_empty() {
        return Ok((VecIndex::Auto, rest));
    }

    // Parse numeric index
    let idx = idx_str
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("'{idx_str}' is not a valid array index"))?;

    Ok((VecIndex::Explicit(idx), rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_index_explicit() {
        assert_eq!(parse_vec_index("[0]").unwrap(), (VecIndex::Explicit(0), ""));
        assert_eq!(parse_vec_index("[1]").unwrap(), (VecIndex::Explicit(1), ""));
        assert_eq!(parse_vec_index("[123]").unwrap(), (VecIndex::Explicit(123), ""));
    }

    #[test]
    fn test_vec_index_explicit_with_rest() {
        assert_eq!(parse_vec_index("[0].name").unwrap(), (VecIndex::Explicit(0), ".name"));
        assert_eq!(parse_vec_index("[5][2].field").unwrap(), (VecIndex::Explicit(5), "[2].field"));
    }

    #[test]
    fn test_vec_index_empty_brackets() {
        assert_eq!(parse_vec_index("[]").unwrap(), (VecIndex::Auto, ""));
        assert_eq!(parse_vec_index("[].name").unwrap(), (VecIndex::Auto, ".name"));
        assert_eq!(parse_vec_index("[][0]").unwrap(), (VecIndex::Auto, "[0]"));
    }

    #[test]
    fn test_vec_index_empty_suffix() {
        assert_eq!(parse_vec_index("").unwrap(), (VecIndex::Auto, ""));
    }

    #[test]
    fn test_vec_index_dot_prefix() {
        assert_eq!(parse_vec_index(".name").unwrap(), (VecIndex::Auto, ".name"));
        assert_eq!(parse_vec_index(".0").unwrap(), (VecIndex::Auto, ".0"));
        assert_eq!(parse_vec_index(".nested.field").unwrap(), (VecIndex::Auto, ".nested.field"));
    }

    #[test]
    fn test_vec_index_invalid_format() {
        let err = parse_vec_index("name").unwrap_err();
        assert!(err.to_string().contains("expected '[index]'"));
    }

    #[test]
    fn test_vec_index_missing_close_bracket() {
        let err = parse_vec_index("[0").unwrap_err();
        assert!(err.to_string().contains("missing closing bracket"));
    }

    #[test]
    fn test_vec_index_non_numeric() {
        let err = parse_vec_index("[abc]").unwrap_err();
        assert!(err.to_string().contains("'abc' is not a valid array index"));
    }

    #[test]
    fn test_vec_index_negative() {
        let err = parse_vec_index("[-1]").unwrap_err();
        assert!(err.to_string().contains("'-1' is not a valid array index"));
    }
}
