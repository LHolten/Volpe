use crate::{
    lexeme_kind::LexemeKind,
    offset::{Offset, Range},
};

// the syntax tree type in this file is meant for use with code highlighting.
// it will also be used as the first step to compilations.
// using an empty error type guarantees a valid syntax tree.

// this is a reference to the source text
#[derive(Clone, Copy)]
pub struct Lexeme<'a> {
    pub kind: LexemeKind,
    pub range: Range<'a>,
}

pub type Semicolon<'a, E> = Vec<Vec<Contained<'a, E>>>;

// the data-structure is as simple as possible but allows code highlighting
// the error type is for missing brackets
pub enum Contained<'a, E> {
    // this annotates a node to be inside brackets
    Brackets {
        brackets: [Result<Lexeme<'a>, E>; 2],
        inner: Semicolon<'a, E>,
    },
    // terminal is a list of lexeme's
    Terminal(Lexeme<'a>),
}

impl<'a, E> Contained<'a, E> {
    pub fn start(&self) -> Option<Offset> {
        match self {
            Contained::Brackets { brackets, inner: _ } => {
                Some(brackets[0].as_ref().ok()?.range.start)
            }
            Contained::Terminal(l) => Some(l.range.start),
        }
    }
    pub fn end(&self) -> Option<Offset> {
        match self {
            Contained::Brackets { brackets, inner: _ } => {
                Some(brackets[1].as_ref().ok()?.range.end)
            }
            Contained::Terminal(l) => Some(l.range.end),
        }
    }
}
