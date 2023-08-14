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
