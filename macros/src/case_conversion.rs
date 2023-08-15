use proc_macro_error::abort;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameCase {
    Snake,
    Kebab,
    Camel,
    Pascal,
    Lower,
    Upper,
}
pub struct InvalidCase;

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
mod tests {
    use super::RenameCase;

    fn test_helper<const N: usize>(case: RenameCase, should_be: &str, variants: [&str; N]) {
        for variant in variants {
            assert_eq!(case.convert_case(variant), should_be);
        }
    }

    // tests for Upper/Lower is written due to lower/upper cases
    // internally are just calls to .to_lowercase/.to_uppercase

    #[test]
    fn test_upper() {
        test_helper(
            RenameCase::Upper,
            "UPPERCASE",
            ["UpperCase", "upperCase", "uppercase", "UPPERCASE"],
        );
    }

    #[test]
    fn test_lower() {
        test_helper(
            RenameCase::Lower,
            "lowercase",
            ["LowerCase", "lowerCase", "lowercase", "LOWERCASE"],
        );
    }

    #[test]
    fn test_kebab() {
        test_helper(
            RenameCase::Kebab,
            "kebab-case",
            ["kebab-case", "KebabCase", "kebab_case", "kebabCase"],
        );
    }

    #[test]
    fn test_pascal() {
        test_helper(
            RenameCase::Pascal,
            "PascalCase",
            ["PascalCase", "pascal_case", "pascal-case", "pascalCase"],
        );
    }

    #[test]
    fn test_camel() {
        test_helper(
            RenameCase::Camel,
            "camelCase",
            ["camelCase", "camel_case", "camel-case", "CamelCase"],
        );
    }
}
