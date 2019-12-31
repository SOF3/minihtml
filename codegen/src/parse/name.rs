use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

#[cfg_attr(test, derive(Debug))]
pub struct Hyphenated {
    pub name: String,
    span: Span,
}

impl AsRef<str> for Hyphenated {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

impl PartialEq<Hyphenated> for Hyphenated {
    fn eq(&self, other: &Self) -> bool {
        &self.name == &other.name
    }
}

impl Eq for Hyphenated {}

impl Hash for Hyphenated {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state)
    }
}

impl Parse for Hyphenated {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let first = input.parse::<syn::Ident>()?;
        let mut name = first.to_string();
        let mut span = first.span();
        while input.peek(syn::Token![-]) {
            let hyphen = input.parse::<syn::Token![-]>().unwrap();
            name.push('-');
            span = span.join(hyphen.span()).unwrap_or(span);
            let ident = input.parse::<syn::Ident>()?;
            name.push_str(&ident.to_string());
            span = span.join(ident.span()).unwrap_or(span);
        }
        Ok(Hyphenated { name, span })
    }
}

impl Spanned for Hyphenated {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_quote::quote;

    #[test]
    fn parse_simple_ident() {
        let parsed = syn::parse2::<Hyphenated>(quote!(abc)).unwrap();
        assert_eq!(parsed.name.as_str(), "abc");
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn parse_hyphenated_ident() {
        let parsed = syn::parse2::<Hyphenated>(quote!(abc - def-ghi)).unwrap();
        assert_eq!(parsed.name.as_str(), "abc-def-ghi");
    }

    #[test]
    fn parse_alphanum() {
        let parsed = syn::parse2::<Hyphenated>(quote!(abc - de0 - g2i)).unwrap();
        assert_eq!(parsed.name.as_str(), "abc-de0-g2i");
    }
}
