use std::fmt;

use super::{Result, ToHtmlAttr};

/// Concatenates hardcoded and dynamic classes.
///
/// The first field is a user-provided value.
/// The second field is a static str from the macro.
pub struct ClassConcat<A>(pub A, pub &'static str)
where
    A: AsRef<str>;

impl<A> ToHtmlAttr for ClassConcat<A>
where
    A: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result {
        write!(f, "{}", self.1)?;
        let str = self.0.as_ref();
        if str.len() > 0 {
            write!(f, " {}", str)?;
        }
        Ok(())
    }
}
