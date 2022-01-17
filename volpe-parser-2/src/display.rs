use crate::offset::Range;
use crate::simple::Simple;
use crate::syntax::Contained;
use crate::syntax::Lexeme;
use std::fmt::Write;
use std::fmt::{self, Debug};

impl<E: Debug> Debug for Contained<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Contained::Brackets { brackets, inner } => {
                f.write_str("Brackets ")?;
                f.debug_list()
                    .entry(brackets[0].as_ref().map(as_debug).unwrap_or(&"NoBracket"))
                    .entries(inner)
                    .entry(brackets[1].as_ref().map(as_debug).unwrap_or(&"NoBracket"))
                    .finish()
            }
            Contained::Terminal(lex) => lex.fmt(f),
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

impl<'a> Debug for Simple<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Push(arg0) => {
                f.write_char('(')?;
                for v in arg0 {
                    v.fmt(f)?;
                    f.write_char(' ')?;
                }
                f.write_char(')')
            }
            Self::Pop(arg0) => {
                f.write_char('[')?;
                arg0.fmt(f)?;
                f.write_char(']')
            }
            Self::Ident(arg0) => arg0.fmt(f),
            Self::Raw(arg0) => {
                f.write_str("#{")?;
                arg0.fmt(f)?;
                f.write_char('}')
            }
        }
    }
}

impl<'a> Debug for Range<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}
