use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

#[cfg_attr(test, derive(Debug))]
pub enum Attribute {
    Static(StaticAttribute),
    Dyn(DynAttribute),
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![dyn]) {
            Ok(Self::Dyn(input.parse()?))
        } else {
            Ok(Self::Static(input.parse()?))
        }
    }
}

impl Spanned for Attribute {
    fn span(&self) -> Span {
        match self {
            Self::Static(sa) => sa.span(),
            Self::Dyn(dyn_) => dyn_.span(),
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct StaticAttribute {
    pub name: AttributeName,
    pub value: Option<(syn::Token![=], syn::Expr)>,
}

impl Parse for StaticAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let value = if input.peek(syn::Token![=]) {
            Some((input.parse().unwrap(), input.parse()?))
        } else {
            None
        };
        Ok(Self { name, value })
    }
}

impl Spanned for StaticAttribute {
    fn span(&self) -> Span {
        let mut span = self.name.span();
        if let Some((eq, expr)) = &self.value {
            span = span.join(eq.span()).unwrap_or(span);
            span = span.join(expr.span()).unwrap_or(span);
        }
        span
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct DynAttribute {
    pub dyn_: syn::Token![dyn],
    pub name: syn::Expr,
    pub value: Option<(syn::Token![=], syn::Expr)>,
}

impl Parse for DynAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dyn_ = input.parse()?;
        let name = input.parse()?;
        let value = if input.peek(syn::Token![=]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };
        Ok(DynAttribute { dyn_, name, value })
    }
}

impl Spanned for DynAttribute {
    fn span(&self) -> Span {
        let mut span = self.name.span();
        span = self.dyn_.span().join(span).unwrap_or(span);
        if let Some((eq, expr)) = &self.value {
            span = span.join(eq.span()).unwrap_or(span);
            span = span.join(expr.span()).unwrap_or(span);
        }
        span
    }
}

pub type AttributeName = super::Hyphenated;

#[cfg(test)]
mod tests {
    use matches2::unwrap_match;
    use proc_quote::quote;

    use super::*;

    #[test]
    fn parse_static_with_value() {
        let parsed = syn::parse2::<Attribute>(quote!(a - b - c = 3)).unwrap();
        let attr = unwrap_match!(parsed, Attribute::Static(x) => x);
        assert_eq!(attr.name.as_ref(), "a-b-c");
        let value = attr.value.unwrap().1;
        assert_eq!(quote!(#value).to_string(), quote!(3).to_string());
    }

    #[test]
    fn parse_static_without_value() {
        let parsed = syn::parse2::<Attribute>(quote!(a - b - c)).unwrap();
        let attr = unwrap_match!(parsed, Attribute::Static(x) => x);
        assert_eq!(attr.name.as_ref(), "a-b-c");
        assert!(attr.value.is_none());
    }

    #[test]
    fn parse_dyn_with_value() {
        let parsed = syn::parse2::<Attribute>(quote!(dyn a+b = c+d)).unwrap();
        let DynAttribute { name, value, .. } = unwrap_match!(parsed, Attribute::Dyn(x) => x);
        assert_eq!(quote!(#name).to_string(), quote!(a + b).to_string());
        let (_, value) = value.unwrap();
        assert_eq!(quote!(#value).to_string(), quote!(c + d).to_string());
    }

    #[test]
    fn parse_dyn_without_value() {
        let parsed = syn::parse2::<Attribute>(quote!(dyn a + b)).unwrap();
        let DynAttribute { name, value, .. } = unwrap_match!(parsed, Attribute::Dyn(x) => x);
        assert_eq!(quote!(#name).to_string(), quote!(a + b).to_string());
        assert!(value.is_none());
    }
}
