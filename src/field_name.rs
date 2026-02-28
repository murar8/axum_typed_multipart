//! Parsing of nested field name notation.
//!
//! Both bracket (`[key]`) and dot (`.key`) notation are supported
//! interchangeably and can be mixed freely. Empty brackets (`[]`) represent an
//! append operation and are only valid as the final segment.

/// A single segment of a parsed field name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment<'a> {
    /// A named key (e.g. `foo` in `foo.bar` or `[foo]`).
    Key(&'a str),
    /// An empty bracket `[]`, only valid as the last segment.
    Append,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input string is empty.
    #[error("field name is empty")]
    Empty,
    /// A segment is empty (e.g. `user..name`, `user[].name`).
    #[error("empty segment at position {position}")]
    EmptySegment { position: usize },
    /// A `[` was not closed with a matching `]`.
    #[error("unclosed bracket at position {position}")]
    UnclosedBracket { position: usize },
    /// A `]` appeared without a preceding `[`.
    #[error("unexpected closing bracket at position {position}")]
    UnexpectedClosingBracket { position: usize },
    /// Content follows `]` without a `.` or `[` separator.
    #[error("missing separator at position {position}")]
    MissingSeparator { position: usize },
}

enum State {
    Key { start: usize },
    Bracket { open: usize },
    Separator { append_pos: Option<usize> },
}

fn next_state(b: u8, i: usize) -> State {
    match b {
        b'.' => State::Key { start: i + 1 },
        b'[' => State::Bracket { open: i },
        _ => unreachable!(),
    }
}

fn step<'a>(
    state: State,
    b: u8,
    i: usize,
    input: &'a str,
    segments: &mut Vec<Segment<'a>>,
) -> Result<State, ParseError> {
    match state {
        State::Separator { append_pos } => {
            if b == b']' {
                Err(ParseError::UnexpectedClosingBracket { position: i })
            } else if b != b'.' && b != b'[' {
                Err(ParseError::MissingSeparator { position: i })
            } else if let Some(pos) = append_pos {
                Err(ParseError::EmptySegment { position: pos })
            } else {
                Ok(next_state(b, i))
            }
        }
        State::Key { start } => {
            if b != b'.' && b != b'[' && b != b']' {
                Ok(state)
            } else if start == i {
                Err(ParseError::EmptySegment { position: i })
            } else if b == b']' {
                Err(ParseError::UnexpectedClosingBracket { position: i })
            } else {
                segments.push(Segment::Key(&input[start..i]));
                Ok(next_state(b, i))
            }
        }
        State::Bracket { open } => {
            if b == b'[' {
                Err(ParseError::UnclosedBracket { position: open })
            } else if b != b']' {
                Ok(state)
            } else {
                let content_start = open + 1;
                if content_start == i {
                    segments.push(Segment::Append);
                    Ok(State::Separator { append_pos: Some(content_start) })
                } else {
                    segments.push(Segment::Key(&input[content_start..i]));
                    Ok(State::Separator { append_pos: None })
                }
            }
        }
    }
}

fn finalize<'a>(
    state: State,
    input: &'a str,
    segments: &mut Vec<Segment<'a>>,
) -> Result<(), ParseError> {
    match state {
        State::Separator { .. } => Ok(()),
        State::Bracket { open } => Err(ParseError::UnclosedBracket { position: open }),
        State::Key { start } => {
            if start == input.len() {
                Err(ParseError::EmptySegment { position: start })
            } else {
                segments.push(Segment::Key(&input[start..]));
                Ok(())
            }
        }
    }
}

fn parse_raw(input: &str) -> Result<Vec<Segment<'_>>, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut segments = Vec::new();
    let mut state = State::Key { start: 0 };

    for (i, &b) in input.as_bytes().iter().enumerate() {
        state = step(state, b, i, input, &mut segments)?;
    }

    finalize(state, input, &mut segments)?;

    Ok(segments)
}

/// Parse a field name into segments.
///
/// Both `.` and `[...]` are treated as interchangeable segment separators.
/// `[]` (append) is only valid as the final segment.
pub fn parse(input: &str) -> Result<Vec<Segment<'_>>, crate::TypedMultipartError> {
    parse_raw(input).map_err(|source| crate::TypedMultipartError::InvalidFieldName {
        field_name: input.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::{ParseError::*, Segment::*, *};

    /// Shorthand: `ok("input", &[...])` asserts `parse_raw` succeeds with the
    /// expected segments.
    fn ok(input: &str, expected: &[Segment<'_>]) {
        assert_eq!(parse_raw(input).unwrap(), expected, "input: {input:?}");
    }

    /// Shorthand: `err("input", ParseError::...)` asserts `parse_raw` fails
    /// with the expected error.
    fn err(input: &str, expected: ParseError) {
        assert_eq!(parse_raw(input).unwrap_err(), expected, "input: {input:?}");
    }

    #[test]
    fn single_segment() {
        ok("name", &[Key("name")]);
        ok("0", &[Key("0")]);
    }

    #[test]
    fn bracket_notation() {
        ok("user[address]", &[Key("user"), Key("address")]);
        ok("user[address][street]", &[Key("user"), Key("address"), Key("street")]);
        ok("items[0]", &[Key("items"), Key("0")]);
    }

    #[test]
    fn dot_notation() {
        ok("user.address", &[Key("user"), Key("address")]);
        ok("user.address.street", &[Key("user"), Key("address"), Key("street")]);
    }

    #[test]
    fn mixed_notation() {
        ok("user[address].street", &[Key("user"), Key("address"), Key("street")]);
        ok("user.address[street]", &[Key("user"), Key("address"), Key("street")]);
        ok("items[0].name", &[Key("items"), Key("0"), Key("name")]);
        ok("pets.0[name]", &[Key("pets"), Key("0"), Key("name")]);
    }

    #[test]
    fn notations_are_interchangeable() {
        let expected = &[Key("user"), Key("address"), Key("street")];
        ok("user[address][street]", expected);
        ok("user.address.street", expected);
        ok("user[address].street", expected);
        ok("user.address[street]", expected);
    }

    #[test]
    fn append_at_end() {
        ok("items[]", &[Key("items"), Append]);
        ok("user[pets][]", &[Key("user"), Key("pets"), Append]);
        ok("user.pets[]", &[Key("user"), Key("pets"), Append]);
    }

    #[test]
    fn append_not_at_end() {
        err("items[][name]", EmptySegment { position: 6 });
        err("items[].name", EmptySegment { position: 6 });
    }

    #[test]
    fn empty_input() {
        err("", Empty);
    }

    #[test]
    fn empty_segments() {
        err(".name", EmptySegment { position: 0 });
        err("user.", EmptySegment { position: 5 });
        err("user..name", EmptySegment { position: 5 });
        err("[name]", EmptySegment { position: 0 });
    }

    #[test]
    fn unclosed_bracket() {
        err("user[address", UnclosedBracket { position: 4 });
    }

    #[test]
    fn unexpected_closing_bracket() {
        err("user]", UnexpectedClosingBracket { position: 4 });
    }

    #[test]
    fn missing_separator_after_bracket() {
        err("user[addr]street", MissingSeparator { position: 10 });
    }

    // ── Deep nesting ────────────────────────────────────────────────

    #[test]
    fn deeply_nested() {
        ok("a[b][c][d].e.f", &[Key("a"), Key("b"), Key("c"), Key("d"), Key("e"), Key("f")]);
    }

    // ── Special characters in keys ──────────────────────────────────

    #[test]
    fn keys_with_special_characters() {
        ok("my-field", &[Key("my-field")]);
        ok("my_field", &[Key("my_field")]);
        ok("field[with-dash]", &[Key("field"), Key("with-dash")]);
        ok("field[with_underscore]", &[Key("field"), Key("with_underscore")]);
        ok("field[with space]", &[Key("field"), Key("with space")]);
    }

    // ── Bracket edge cases ──────────────────────────────────────────

    #[test]
    fn nested_open_bracket() {
        err("user[addr[inner]]", UnclosedBracket { position: 4 });
    }

    #[test]
    fn closing_bracket_inside_bare() {
        err("user]name", UnexpectedClosingBracket { position: 4 });
    }

    #[test]
    fn closing_bracket_after_dot() {
        err("user.]name", EmptySegment { position: 5 });
    }

    #[test]
    fn unclosed_bracket_at_end() {
        err("user[", UnclosedBracket { position: 4 });
    }

    #[test]
    fn only_brackets() {
        err("[]", EmptySegment { position: 0 });
    }

    #[test]
    fn dot_after_closed_bracket_no_key() {
        err("user[addr].", EmptySegment { position: 11 });
    }

    // ── Append edge cases ───────────────────────────────────────────

    #[test]
    fn bare_name_then_append() {
        ok("x[]", &[Key("x"), Append]);
    }

    #[test]
    fn double_append() {
        err("items[][]", EmptySegment { position: 6 });
    }

    // ── Single character keys ───────────────────────────────────────

    #[test]
    fn single_char_segments() {
        ok("a.b.c", &[Key("a"), Key("b"), Key("c")]);
        ok("a[b][c]", &[Key("a"), Key("b"), Key("c")]);
    }
}
