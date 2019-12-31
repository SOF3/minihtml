use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

use crate::ctx;

macro_rules! impl_span {
    ($target:ty = $main:ident $(<< $front:ident)* $(>> $back:ident)*) => {
        impl ::syn::spanned::Spanned for $target {
            fn span(&self) -> proc_macro2::Span {
                use ::syn::spanned::Spanned;
                let mut span = Spanned::span(&self.$main);
                $(
                    span = Spanned::span(&self.$front).join(span).unwrap_or(span);
                )*
                $(
                    span = span.join(Spanned::span(&self.$back)).unwrap_or(span);
                )*
                span
            }
        }
    };
}

mod name;
pub use name::*;

mod attr;
pub use attr::*;

mod element;
pub use element::*;

mod id_class;
pub use id_class::*;

pub struct HtmlNodes {
    pub nodes: Vec<HtmlNode>,
    span: Span,
}

impl Parse for HtmlNodes {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let mut span = input.cursor().span();
        let mut nodes = vec![];
        while !input.is_empty() {
            let node = input
                .parse::<HtmlNode>()
                .map_err(ctx("Parsing HtmlNode in node list"))?;
            span = span.join(node.span()).unwrap_or(span);
            nodes.push(node);
        }
        Ok(Self { nodes, span })
    }
}

impl Spanned for HtmlNodes {
    fn span(&self) -> Span {
        self.span
    }
}

pub enum HtmlNode {
    Arbitrary(syn::Token![+], syn::Expr),
    Element(HtmlElement),
}

impl Parse for HtmlNode {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let ret = if input.peek(syn::Token![+]) {
            let plus: syn::Token![+] = input.parse().unwrap();
            let expr: syn::Expr = input
                .parse()
                .map_err(ctx("Parsing arbitrary HTML node expression"))?;
            if input.peek(syn::Token![;]) {
                input.parse::<syn::Token![;]>().unwrap();
            }
            HtmlNode::Arbitrary(plus, expr)
        } else {
            HtmlNode::Element(input.parse().map_err(ctx("Parsing HTML element node"))?)
        };
        Ok(ret)
    }
}

impl Spanned for HtmlNode {
    fn span(&self) -> Span {
        match self {
            Self::Arbitrary(add, expr) => {
                let mut span = expr.span();
                span = add.span().join(span).unwrap_or(span);
                span
            }
            Self::Element(el) => el.span(),
        }
    }
}
