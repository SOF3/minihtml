use syn::parse::{Parse, ParseStream};

pub struct HashId {
    pub hash: syn::Token![#],
    pub name: IdName,
}

impl Parse for HashId {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            hash: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl_span!(HashId = name << hash);

pub struct DotClass {
    pub dot: syn::Token![.],
    pub name: ClassName,
}

impl Parse for DotClass {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            dot: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl_span!(DotClass = name << dot);

pub type IdName = super::Hyphenated;
pub type ClassName = super::Hyphenated;

#[cfg(test)]
mod tests {
    use proc_quote::quote;

    use super::*;

    #[test]
    fn parse_hash_id() {
        let hash = quote![#];

        let parsed = syn::parse2::<HashId>(quote!(#hash abc-def)).unwrap();
        assert_eq!(parsed.name.as_ref(), "abc-def");
    }

    #[test]
    fn parse_dot_class() {
        let parsed = syn::parse2::<DotClass>(quote!(.abc-def)).unwrap();
        assert_eq!(parsed.name.as_ref(), "abc-def");
    }
}
