extern crate proc_macro as pm1;

use std::collections::HashMap;
use std::fmt;

use proc_macro2::TokenStream;
use proc_quote::quote;
use syn::spanned::Spanned;

mod parse;

#[proc_macro_hack::proc_macro_hack]
pub fn html(input: pm1::TokenStream) -> pm1::TokenStream {
    html_impl(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn ctx<D: fmt::Display>(d: D) -> impl Fn(syn::Error) -> syn::Error {
    move |err| syn::Error::new(err.span(), format!("{}: {}", &d, err))
}

fn html_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let nodes = syn::parse2::<parse::HtmlNodes>(input).map_err(ctx("Parsing HTML input"))?;
    let nodes = nodes
        .nodes
        .into_iter()
        .map(write_node)
        .collect::<syn::Result<Vec<TokenStream>>>()?;
    let result = quote! {{
        let x = |output: &mut ::std::fmt::Formatter| -> ::std::fmt::Result {
            use ::std::fmt;
            use ::std::write;

            #(#nodes)*
            Ok(())
        };

        ::minihtml::Html(x)
    }};
    Ok(result)
}

fn write_node(node: parse::HtmlNode) -> syn::Result<TokenStream> {
    Ok(match node {
        parse::HtmlNode::Arbitrary(_, expr) => {
            quote! {
                ::minihtml::ToHtmlNode::fmt(#expr, output)?;
            }
        }
        parse::HtmlNode::Element(element) => {
            let element_name = element.name.as_ref();
            let write_attrs = write_el_attrs(&element)?;
            let write_child = match element.children {
                Some(inner_nodes) => {
                    let inner_nodes = inner_nodes
                        .nodes
                        .into_iter()
                        .map(write_node)
                        .collect::<syn::Result<Vec<_>>>()?;
                    quote! {
                        write!(output, ">")?;
                        #(#inner_nodes)*
                        write!(output, concat!("</", #element_name, ">"))?;
                    }
                }
                None => quote! {
                    write!(output, "/>")?;
                },
            };
            quote! {
                write!(output, concat!("<", #element_name))?;
                #write_attrs
                #write_child
            }
        }
    })
}

fn write_el_attrs(element: &parse::HtmlElement) -> syn::Result<TokenStream> {
    let mut static_attrs = HashMap::new();
    let mut dyn_attrs = vec![];
    for attr in element.attributes.iter().flat_map(|(_, attr)| attr) {
        match attr {
            parse::Attribute::Static(attr) => {
                if static_attrs.contains_key(attr.name.as_ref()) {
                    return Err(syn::Error::new(
                        attr.span(),
                        &format!("Duplicate attribute \"{}\"", &attr.name.name),
                    ))?;
                }
                static_attrs.insert(
                    attr.name.as_ref().to_string(),
                    attr.value
                        .as_ref()
                        .map_or_else(|| quote!(true), |(_, expr)| quote!(#expr)),
                );
            }
            parse::Attribute::Dyn(attr) => {
                dyn_attrs.push(attr);
            }
        }
    }

    if let Some(id) = &element.id {
        if static_attrs.contains_key("id") {
            return Err(syn::Error::new(
                id.span(),
                "Duplicate definition of attribute \"id\"",
            ));
        }
        let id = id.name.as_ref();
        static_attrs.insert("id".to_string(), quote!(#id));
    }

    if element.classes.len() > 0 {
        use itertools::Itertools;

        let static_classes_joined = element
            .classes
            .iter()
            .map(|class| class.name.as_ref())
            .join(" ");
        if static_attrs.contains_key("class") {
            let dy = static_attrs
                .get_mut("class")
                .expect("Checked in the condition above");
            *dy = quote! {
                ::minihtml::hc::ClassConcat(#dy, #static_classes_joined)
            };
        } else {
            static_attrs.insert(
                "class".to_string(),
                quote! {
                    ::minihtml::NoSpecial(#static_classes_joined)
                },
            );
        }
    }

    let attrs = static_attrs
        .iter()
        .map(|(name, value)| {
            quote! {
                ::minihtml::ToWholeHtmlAttr::fmt(
                    &(#value),
                    ::minihtml::NoSpecial(#name),
                    output
                )?;
            }
        })
        .chain(dyn_attrs.iter().map(|attr| {
            let name = &attr.name;
            let value = match &attr.value {
                Some((_, value)) => quote!(#value),
                None => quote!(true),
            };
            let static_names = static_attrs.iter().map(|(name, _)| name);
            quote! {
                let name = &(#name);
                debug_assert!(match name {
                    #(#static_names)|* => false,
                    _ => true,
                }, "The dynamic attribute {} duplicates a hardcoded attribute", name);
                ::minihtml::ToWholeHtmlAttr::fmt(
                    &(#value),
                    ::minihtml::NoSpecial::debug_checked(name),
                    output
                )?;
            }
        }));

    Ok(quote!(#(#attrs)*))
}
