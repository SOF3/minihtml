use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use super::{Attribute, DotClass, HashId, HtmlNodes};

pub struct HtmlElement {
    pub name: ElementName,
    pub classes: Vec<DotClass>,
    pub id: Option<HashId>,
    pub attributes: Option<(syn::token::Paren, Punctuated<Attribute, syn::Token![,]>)>,
    pub children: Option<HtmlNodes>,
    span: Span,
}

impl Parse for HtmlElement {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        use crate::ctx;

        let name: ElementName = input.parse().map_err(ctx("Parsing element name"))?;
        let mut span = name.span();

        let mut classes = vec![];
        let mut id = None;
        loop {
            if input.peek(syn::Token![.]) {
                let class: DotClass = input.parse()?;
                span = span.join(class.span()).unwrap_or(span);
                classes.push(class);
            } else if input.peek(syn::Token![#]) {
                if id.is_some() {
                    return Err(input.error("An element may only have one ID"))?;
                }
                let hi: HashId = input.parse().map_err(ctx("Parsing element ID"))?;
                span = span.join(hi.span()).unwrap_or(span);
                id = Some(hi);
            } else if input.peek(syn::token::Paren)
                || input.peek(syn::token::Brace)
                || input.peek(syn::Token![;])
                || input.is_empty()
            {
                break;
            } else {
                return Err(input.error("Unexpected token in HtmlElement"))?;
            }
        }

        let attributes = if input.peek(syn::token::Paren) {
            let inner;
            let token = syn::parenthesized!(inner in input);
            span = span.join(token.span).unwrap_or(span);
            Some((
                token,
                inner
                    .parse_terminated(Attribute::parse)
                    .map_err(ctx("Parsing element attribute"))?,
            ))
        } else {
            None
        };

        let children = if input.peek(syn::token::Brace) {
            let inner;
            let _token = syn::braced!(inner in input);
            let nodes: HtmlNodes = inner.parse().map_err(ctx("Parsing inner elements"))?;
            span = span.join(nodes.span()).unwrap_or(span);
            Some(nodes)
        } else {
            None
        };

        if input.peek(syn::Token![;]) {
            input.parse::<syn::Token![;]>().unwrap();
        }

        Ok(HtmlElement {
            name,
            classes,
            id,
            attributes,
            children,
            span,
        })
    }
}

impl Spanned for HtmlElement {
    fn span(&self) -> Span {
        self.span
    }
}

pub type ElementName = super::Hyphenated;

#[cfg(test)]
mod tests {
    // use matches2::unwrap_match;
    use proc_quote::quote;

    use super::*;

    #[test]
    fn test_empty_single() {
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(parsed.classes.len(), 0);
        assert!(parsed.id.is_none());
        assert!(parsed.attributes.is_none());
        assert!(parsed.children.is_none());
    }

    #[test]
    fn test_empty_single_semi() {
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo;
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(parsed.classes.len(), 0);
        assert!(parsed.id.is_none());
        assert!(parsed.attributes.is_none());
        assert!(parsed.children.is_none());
    }

    #[test]
    fn test_empty_block() {
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo {}
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(parsed.classes.len(), 0);
        assert!(parsed.id.is_none());
        assert!(parsed.attributes.is_none());
        assert_eq!(parsed.children.unwrap().nodes.into_iter().count(), 0);
    }

    #[test]
    fn test_empty_block_semi() {
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo {};
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(parsed.classes.len(), 0);
        assert!(parsed.id.is_none());
        assert!(parsed.attributes.is_none());
        assert_eq!(parsed.children.unwrap().nodes.into_iter().count(), 0);
    }

    #[test]
    fn test_id() {
        let hash = quote![#];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo #hash ab-c;
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(parsed.classes.len(), 0);
        assert_eq!(parsed.id.unwrap().name.as_ref(), "ab-c");
        assert!(parsed.attributes.is_none());
        assert!(parsed.children.is_none());
    }

    #[test]
    fn test_classes() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo.de-f.gh-i;
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(
            parsed
                .classes
                .iter()
                .map(|class| class.name.as_ref().to_string())
                .collect::<Vec<String>>(),
            vec!["de-f".to_string(), "gh-i".to_string()]
        );
        assert!(parsed.id.is_none());
        assert!(parsed.attributes.is_none());
        assert!(parsed.children.is_none());
    }

    #[test]
    fn test_classes_num() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo.de-f0.gh-i;
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(
            parsed
                .classes
                .iter()
                .map(|class| class.name.as_ref().to_string())
                .collect::<Vec<String>>(),
            vec!["de-f0".to_string(), "gh-i".to_string()]
        );
        assert!(parsed.id.is_none());
        assert!(parsed.attributes.is_none());
        assert!(parsed.children.is_none());
    }

    #[test]
    fn test_id_classes_mixed() {
        let hash = quote![#];
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let parsed = syn::parse2::<HtmlElement>(quote! {
            foo .de-f #hash ab-c .gh-i;
        })
        .unwrap();
        assert_eq!(parsed.name.as_ref(), "foo");
        assert_eq!(
            parsed
                .classes
                .iter()
                .map(|class| class.name.as_ref().to_string())
                .collect::<Vec<String>>(),
            vec!["de-f".to_string(), "gh-i".to_string()]
        );
        assert_eq!(parsed.id.unwrap().name.as_ref(), "ab-c");
        assert!(parsed.attributes.is_none());
        assert!(parsed.children.is_none());
    }
}
