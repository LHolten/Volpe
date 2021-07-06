use crate::{lexeme_kind::LexemeKind, offset::Offset};

// the syntax tree type in this file is meant for use with code highlighting.
// it will also be used as the first step to compilations.
// using an empty error type guarantees a valid syntax tree.

// this is a reference to the source text
#[derive(Debug, Clone, Copy)]
pub struct Lexeme<'a> {
    pub start: Offset,
    pub end: Offset,
    pub kind: LexemeKind,
    pub text: &'a str,
}

// the data-structure is as simple as possible but allows code highlighting
#[derive(Debug)]
pub enum Syntax<'a, E> {
    // this applies an operator to two nodes
    Operator {
        operator: Option<Lexeme<'a>>, // if this is none then it is an application
        operands: [Box<Syntax<'a, E>>; 2],
    },
    // this annotates a node to be inside brackets
    Brackets {
        brackets: [Result<Lexeme<'a>, E>; 2],
        inner: Box<Syntax<'a, E>>,
    },
    // terminal is just a single self contained lexeme
    Terminal(Result<Lexeme<'a>, E>),
}
