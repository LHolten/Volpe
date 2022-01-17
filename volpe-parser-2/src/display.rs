use crate::offset::Range;
use crate::simple::Simple;
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
            Semicolon::Semi { left, semi, right } => {
                let mut res = left.iter().map(as_debug).collect::<Vec<_>>();
                res.push(semi);
                res.extend(right.get_list());
                res
            }
            Semicolon::Syntax(inner) => inner.iter().map(as_debug).collect::<Vec<_>>(),
        }
    }
}

impl<E: Debug> Debug for Semicolon<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get_list().fmt(f)
    }
}

fn as_debug<T: Debug>(val: &'_ T) -> &'_ dyn Debug {
    val
}

impl Debug for Lexeme<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        f.write_char(' ')?;
        self.range.text.fmt(f)
    }
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
