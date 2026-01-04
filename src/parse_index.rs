/// Parses `[index]` from the start of the suffix.
///
/// Returns:
/// - `Ok((idx, rest))` on successful parse
/// - `Err(...)` if index format is invalid or missing
pub(crate) fn parse_index(suffix: &str) -> Result<(usize, &str), anyhow::Error> {
    let rest = suffix
        .strip_prefix('[')
        .ok_or_else(|| anyhow::anyhow!("expected '[index]' but found '{suffix}'"))?;
    let end = rest.find(']').ok_or_else(|| anyhow::anyhow!("missing closing bracket"))?;
    let idx_str = &rest[..end];
    let idx = idx_str
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("'{idx_str}' is not a valid number"))?;
    Ok((idx, &rest[end + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_index() {
        assert_eq!(parse_index("[0]").unwrap(), (0, ""));
        assert_eq!(parse_index("[1]").unwrap(), (1, ""));
        assert_eq!(parse_index("[123]").unwrap(), (123, ""));
    }

    #[test]
    fn test_valid_index_with_rest() {
        assert_eq!(parse_index("[0].name").unwrap(), (0, ".name"));
        assert_eq!(parse_index("[5][2].field").unwrap(), (5, "[2].field"));
        assert_eq!(parse_index("[0]rest").unwrap(), (0, "rest"));
    }

    #[test]
    fn test_missing_open_bracket() {
        let err = parse_index("0]").unwrap_err();
        assert!(err.to_string().contains("expected '[index]'"));
    }

    #[test]
    fn test_empty_string() {
        let err = parse_index("").unwrap_err();
        assert!(err.to_string().contains("expected '[index]'"));
    }

    #[test]
    fn test_missing_close_bracket() {
        let err = parse_index("[0").unwrap_err();
        assert!(err.to_string().contains("missing closing bracket"));
    }

    #[test]
    fn test_empty_brackets() {
        let err = parse_index("[]").unwrap_err();
        assert!(err.to_string().contains("not a valid number"));
    }

    #[test]
    fn test_non_numeric() {
        let err = parse_index("[abc]").unwrap_err();
        assert!(err.to_string().contains("'abc' is not a valid number"));
    }

    #[test]
    fn test_negative_index() {
        let err = parse_index("[-1]").unwrap_err();
        assert!(err.to_string().contains("'-1' is not a valid number"));
    }

    #[test]
    fn test_float_index() {
        let err = parse_index("[1.5]").unwrap_err();
        assert!(err.to_string().contains("'1.5' is not a valid number"));
    }

    #[test]
    fn test_whitespace_in_index() {
        // Leading/trailing whitespace is invalid
        let err = parse_index("[ 0]").unwrap_err();
        assert!(err.to_string().contains("not a valid number"));
    }

    #[test]
    fn test_dot_prefix() {
        let err = parse_index(".name").unwrap_err();
        assert!(err.to_string().contains("expected '[index]'"));
    }
}
