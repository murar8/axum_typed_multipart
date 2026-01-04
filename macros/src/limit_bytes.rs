use quote::{quote, ToTokens};
use ubyte::ByteUnit;

/// Parsed byte limit from attribute (e.g., `limit = "1MB"` or `limit = "unlimited"`).
/// `None` represents unlimited.
#[derive(Debug, Clone, Copy, Default)]
pub struct LimitBytes(Option<ByteUnit>);

impl darling::FromMeta for LimitBytes {
    fn from_string(value: &str) -> darling::Result<Self> {
        if value == "unlimited" {
            Ok(Self(None))
        } else {
            value.parse::<ByteUnit>().map(|b| Self(Some(b))).map_err(|_| {
                darling::Error::custom("must be a valid byte unit (e.g., \"1MB\", \"500KB\")")
            })
        }
    }
}

impl ToTokens for LimitBytes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self.0 {
            None => quote! { None },
            Some(b) => {
                let n = b.as_u64() as usize;
                quote! { Some(#n) }
            }
        });
    }
}

impl LimitBytes {
    pub fn is_unlimited(&self) -> bool {
        self.0.is_none()
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use darling::FromMeta;

    #[test]
    fn test_valid_units() {
        assert_eq!(LimitBytes::from_string("1MB").unwrap().0, Some(ByteUnit::Megabyte(1)));
        assert_eq!(LimitBytes::from_string("500KB").unwrap().0, Some(ByteUnit::Kilobyte(500)));
    }

    #[test]
    fn test_unlimited() {
        assert!(LimitBytes::from_string("unlimited").unwrap().0.is_none());
    }

    #[test]
    fn test_invalid() {
        let err = LimitBytes::from_string("invalid").unwrap_err();
        assert!(err.to_string().contains("must be a valid byte unit"));
    }
}
