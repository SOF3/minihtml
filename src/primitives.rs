use std::fmt;

use super::{Escaped, NoSpecial, Result, ToHtmlAttr, ToHtmlNode, ToWholeHtmlAttr};

impl<T: ToHtmlNode + ?Sized> ToHtmlNode for &T {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        ToHtmlNode::fmt(&**self, f)
    }
}

impl<T: ToHtmlAttr + ?Sized> ToHtmlAttr for &T {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        ToHtmlAttr::fmt(&**self, f)
    }
}

impl ToHtmlNode for str {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        write!(f, "{}", Escaped(self.as_ref()))
    }
}

impl ToHtmlAttr for str {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        write!(f, "{}", Escaped(self.as_ref()))
    }
}

impl ToWholeHtmlAttr for bool {
    #[inline]
    fn fmt(&self, name: NoSpecial<'_>, f: &mut fmt::Formatter) -> Result {
        if *self {
            write!(f, " {}", name.0)?;
        }
        Ok(())
    }
}

impl<T: ToHtmlAttr> ToWholeHtmlAttr for Option<T> {
    #[inline]
    fn fmt(&self, name: NoSpecial<'_>, f: &mut fmt::Formatter) -> Result {
        if let Some(value) = self {
            write!(f, " {}=\"", name.0)?;
            ToHtmlAttr::fmt(value, f)?;
            write!(f, "\"")?;
        }
        Ok(())
    }
}
