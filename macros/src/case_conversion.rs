use proc_macro_error::abort;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameCase {
    Snake,
    Kebab,
    Camel,
    Pascal,
    Lower,
    Upper,
}

#[derive(Debug, Clone)]
pub struct InvalidCase;

impl Display for InvalidCase {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "invalid case conversion option")
    }
}

impl Error for InvalidCase {}

impl RenameCase {
    pub fn convert_case(self, s: &str) -> String {
        match self {
            Self::Snake => format!("{}", heck::AsSnakeCase(s)),
            Self::Camel => format!("{}", heck::AsLowerCamelCase(s)),
            Self::Kebab => format!("{}", heck::AsKebabCase(s)),
            Self::Pascal => format!("{}", heck::AsPascalCase(s)),
            Self::Lower => s.to_lowercase(),
            Self::Upper => s.to_uppercase(),
        }
    }

    pub fn from_option_fallible<S>(spanned: S, option: Option<impl AsRef<str>>) -> Option<Self>
    where
        S: darling::ToTokens,
    {
        option.as_ref().map(|r| r.as_ref()).map(|value| {
            Self::try_from(value)
                .unwrap_or_else(|_| abort!(spanned, "invalid case conversion option"))
        })
    }
}

impl<'a> TryFrom<&'a str> for RenameCase {
    type Error = InvalidCase;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "snake_case" => Ok(Self::Snake),
            "kebab-case" => Ok(Self::Kebab),
            "camelCase" => Ok(Self::Camel),
            "PascalCase" => Ok(Self::Pascal),
            "UPPERCASE" => Ok(Self::Upper),
            "lowercase" => Ok(Self::Lower),
            _ => Err(InvalidCase),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    fn test_convert_case<const N: usize>(case: RenameCase, should_be: &str, variants: [&str; N]) {
        for variant in variants {
            assert_eq!(case.convert_case(variant), should_be);
        }
    }

    #[test]
    fn test_convert_case_upper() {
        test_convert_case(
            RenameCase::Upper,
            "UPPERCASE",
            ["UpperCase", "upperCase", "uppercase", "UPPERCASE"],
        );
    }

    #[test]
    fn test_convert_case_snake() {
        test_convert_case(
            RenameCase::Snake,
            "snake_case",
            ["snake_case", "SnakeCase", "snake-case", "snakeCase"],
        );
    }

    #[test]
    fn test_convert_case_lower() {
        test_convert_case(
            RenameCase::Lower,
            "lowercase",
            ["LowerCase", "lowerCase", "lowercase", "LOWERCASE"],
        );
    }

    #[test]
    fn test_convert_case_kebab() {
        test_convert_case(
            RenameCase::Kebab,
            "kebab-case",
            ["kebab-case", "KebabCase", "kebab_case", "kebabCase"],
        );
    }

    #[test]
    fn test_convert_case_pascal() {
        test_convert_case(
            RenameCase::Pascal,
            "PascalCase",
            ["PascalCase", "pascal_case", "pascal-case", "pascalCase"],
        );
    }

    #[test]
    fn test_convert_case_camel() {
        test_convert_case(
            RenameCase::Camel,
            "camelCase",
            ["camelCase", "camel_case", "camel-case", "CamelCase"],
        );
    }

    #[test]
    fn test_try_from_valid() {
        assert_eq!(RenameCase::try_from("snake_case").unwrap(), RenameCase::Snake);
        assert_eq!(RenameCase::try_from("kebab-case").unwrap(), RenameCase::Kebab);
        assert_eq!(RenameCase::try_from("camelCase").unwrap(), RenameCase::Camel);
        assert_eq!(RenameCase::try_from("PascalCase").unwrap(), RenameCase::Pascal);
        assert_eq!(RenameCase::try_from("UPPERCASE").unwrap(), RenameCase::Upper);
        assert_eq!(RenameCase::try_from("lowercase").unwrap(), RenameCase::Lower);
    }

    #[test]
    fn test_try_from_err() {
        let error = RenameCase::try_from("invalid_case").unwrap_err();
        assert!(matches!(error, InvalidCase));
        assert_eq!(error.to_string(), "invalid case conversion option");
    }

    #[test]
    fn test_from_option_fallible() {
        assert_eq!(RenameCase::from_option_fallible(0, None::<String>), None);
        assert_eq!(
            RenameCase::from_option_fallible("", Some("snake_case")),
            Some(RenameCase::Snake)
        );
    }
}
