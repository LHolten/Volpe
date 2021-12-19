use crate::syntax::Contained;
use crate::syntax::Lexeme;
use crate::syntax::Semicolon;
use std::fmt::Write;
use std::fmt::{self, Debug};

impl<E: Debug> Debug for Contained<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Contained::Brackets { brackets, inner } => {
                f.write_str("Brackets ")?;
                f.debug_list()
                    .entry(brackets[0].as_ref().map(as_debug).unwrap_or(&"NoBracket"))
                    .entries(inner.get_list())
                    .entry(brackets[1].as_ref().map(as_debug).unwrap_or(&"NoBracket"))
                    .finish()
            }
            Contained::Terminal(lex) => lex.fmt(f),
        }
    }
}

impl<E: Debug> Semicolon<'_, E> {
    fn get_list(&self) -> Vec<&dyn Debug> {
        match self {
            Semicolon::Semi {
                left,
                semi: _,
                right,
            } => {
                let mut res = left.iter().map(as_debug).collect::<Vec<_>>();
                res.push(right);
                res
            }
            Semicolon::Syntax(inner) => inner.iter().map(as_debug).collect::<Vec<_>>(),
        }
    }
}
impl<E: Debug> Debug for Semicolon<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Semicolon::Semi {
                left: _,
                semi: _,
                right: _,
            } => {
                f.write_str("Semi ")?;
                f.debug_list().entries(self.get_list()).finish()
            }
            Semicolon::Syntax(_) => {
                f.write_str("Syntax ")?;
                f.debug_list().entries(self.get_list()).finish()
            }
        }
    }
}

impl Debug for Lexeme<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        f.write_char(' ')?;
        self.range.text.fmt(f)
    }
}

fn as_debug<T: Debug>(val: &'_ T) -> &'_ dyn Debug {
    val
}
