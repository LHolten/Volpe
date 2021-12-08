use std::error::Error;
use std::fmt;

use crate::offset::Offset;
use crate::syntax::Lexeme;

#[derive(Debug)]
pub enum PatchError {
    OffsetOutOfRange,
    LengthOutOfRange,
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatchError::OffsetOutOfRange => write!(f, "Offset is out of range"),
            PatchError::LengthOutOfRange => write!(f, "Length is out of range"),
        }
    }
}

impl Error for PatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub enum SyntaxError<'a> {
    UnmatchedBracket(Lexeme<'a>),
    MismatchedBracket(Lexeme<'a>),
}

impl<'a> SyntaxError<'a> {
    pub fn get_range(&self) -> (Offset, Offset) {
        match self {
            SyntaxError::UnmatchedBracket(lexeme) => (lexeme.start, lexeme.end),
            SyntaxError::MismatchedBracket(lexeme) => (lexeme.start, lexeme.end),
        }
    }
}

impl<'a> fmt::Display for SyntaxError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::UnmatchedBracket(_) => write!(f, "Missing bracket"),
            SyntaxError::MismatchedBracket(_) => write!(f, "Mismatched bracket"),
        }
    }
}

impl<'a> Error for SyntaxError<'a> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
