use std::fmt;

pub type Result<T = (), E = fmt::Error> = std::result::Result<T, E>;

/// A Node is either an element or a text string
pub trait ToHtmlNode {
    /// Writes the content of the node.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result;
}

/// Writes an attribute, where the name is given and the value is `self`.
///
/// Prefer implementing `ToHtmlAttr` over this trait if your implementation always has a name and a
/// value.
/// This trait should only be directly implemented for special attribute types like `bool` and
/// `Option`, which optionally writes the attribute name or does not have a value.
pub trait ToWholeHtmlAttr {
    /// Writes the whole attribute, **including the leading space before the attribute name**, to the
    /// output.
    fn fmt(&self, name: NoSpecial<'_>, f: &mut fmt::Formatter) -> Result;
}

/// Writes an attribute value.
///
/// This trait comes with a blanket impl for `ToWholeHtmlAttr`.
/// Types that do not always have a value should implement `ToWholeHtmlAttr` directly.
pub trait ToHtmlAttr {
    /// Writes the attribute value.
    ///
    /// It is the responsibility of the implementor to make sure that no `"` characters are written
    /// during this function call.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result;
}

impl<T: ToHtmlAttr + ?Sized> ToWholeHtmlAttr for T {
    #[inline]
    fn fmt(&self, name: NoSpecial<'_>, f: &mut fmt::Formatter) -> Result {
        write!(f, " {}=\"", name.0)?;
        ToHtmlAttr::fmt(self, f)?;
        write!(f, "\"")?;
        Ok(())
    }
}

/// Indicates that the wrapped value is guaranteed to have no special characters.
///
/// Do NOT use this struct to indicate that a string should not be escaped in text mode;
/// use `Raw` for that case.
///
/// "No special characters" means absolutely none, no matter escaped or not. (`&amp;` still
/// contains the special character `&`)
#[derive(Debug, Clone, Copy)]
pub struct NoSpecial<'t>(pub &'t str);

impl<'t> NoSpecial<'t> {
    #[inline]
    pub fn debug_checked(value: &'t str) -> Self {
        debug_assert!(
            !has_special_chars(value),
            "Value passed to NoSpecial::debug_checked() contains speicla characters: {:?}",
            value
        );
        Self(value)
    }
}

fn has_special_chars(s: &str) -> bool {
    s.contains(|c| match c {
        '&' | '<' | '>' | '\'' | '"' => true,
        _ => false,
    })
}

impl<'t> ToHtmlNode for NoSpecial<'t> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}

impl<'t> ToHtmlAttr for NoSpecial<'t> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}

struct Escaped<'t>(&'t str);

impl<'t> fmt::Display for Escaped<'t> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        for char in self.0.chars() {
            let escape = match char {
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                '\'' => "&apos;",
                '"' => "&quot;",
                _ => {
                    write!(f, "{}", char)?;
                    continue;
                }
            };
            write!(f, "{}", escape)?;
        }
        Ok(())
    }
}

mod primitives;

#[proc_macro_hack::proc_macro_hack]
pub use minihtml_codegen::html;

#[doc(hidden)]
pub struct HtmlString<T: ToHtmlNode>(pub T);

impl<T: ToHtmlNode> fmt::Display for HtmlString<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        ToHtmlNode::fmt(&self.0, f)
    }
}

#[doc(hidden)]
pub struct Html<F>(pub F)
where
    F: Fn(&mut ::std::fmt::Formatter) -> fmt::Result;

impl<F> ToHtmlNode for Html<F>
where
    F: Fn(&mut ::std::fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

/// Returns the string containing the HTML strings.
///
/// The return type is `Result<std::string::String, std::fmt::Error>`.
#[macro_export]
macro_rules! html_string {
    ($($tt:tt)*) => {{
        let node = $crate::html!($($tt)*);
        format!("{}", $crate::HtmlString(node))
    }}
}

/// Symbols used in codegen output hardcoding
#[doc(hidden)]
pub mod hc;
