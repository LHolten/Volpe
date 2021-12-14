use crate::{
    lexeme_kind::LexemeKind,
    offset::{Offset, Range},
};

// the syntax tree type in this file is meant for use with code highlighting.
// it will also be used as the first step to compilations.
// using an empty error type guarantees a valid syntax tree.

// this is a reference to the source text
#[derive(Debug, Clone, Copy)]
pub struct Lexeme<'a> {
    pub kind: LexemeKind,
    pub range: Range<'a>,
}

// the data-structure is as simple as possible but allows code highlighting
// the error type is for missing brackets
#[derive(Debug)]
pub enum Semicolon<'a, E> {
    Semi {
        left: Vec<Contained<'a, E>>,
        semi: Lexeme<'a>,
        right: Box<Semicolon<'a, E>>,
    },
    Syntax(Vec<Contained<'a, E>>),
}

#[derive(Debug)]
pub enum Contained<'a, E> {
    // this annotates a node to be inside brackets
    Brackets {
        brackets: [Result<Lexeme<'a>, E>; 2],
        inner: Box<Semicolon<'a, E>>,
    },
    // terminal is a list of lexeme's
    Terminal(Lexeme<'a>),
}

impl<'a, E> Contained<'a, E> {
    pub fn start(&self) -> Option<Offset> {
        match self {
            Contained::Brackets { brackets, inner: _ } => {
                brackets[0].as_ref().map(|l| l.range.start).ok()
            }
            Contained::Terminal(l) => Some(l.range.start),
        }
    }
    pub fn end(&self) -> Option<Offset> {
        match self {
            Contained::Brackets { brackets, inner: _ } => {
                brackets[1].as_ref().map(|l| l.range.end).ok()
            }
            Contained::Terminal(l) => Some(l.range.end),
        }
    }
}
