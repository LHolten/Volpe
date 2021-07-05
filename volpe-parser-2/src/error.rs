use std::error::Error;
use std::fmt;

use crate::lexeme_kind::LexemeKind;
use crate::offset::Offset;
use crate::syntax::Syntax;

#[derive(Debug)]
pub enum PatchError {
    OffsetOutOfRange,
    LengthOutOfRange,
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PatchError::OffsetOutOfRange => "Offset is out of range",
                PatchError::LengthOutOfRange => "Length is out of range",
            }
        )
    }
}

impl Error for PatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub enum SyntaxError {
    MissingTerminal(Offset),
    MissingBracket(Offset),
    MismatchedBracket(Offset, LexemeKind),
    // MisplacedOperator(Offset)
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SyntaxError::MissingTerminal(_) => "Missing terminal",
                SyntaxError::MissingBracket(_) => "Missing bracket",
                SyntaxError::MismatchedBracket(_, _) => "Mismatched bracket kind",
            }
        )
    }
}

impl Error for SyntaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl<'a> Syntax<'a, ()> {
    pub fn iter_errs(&self) -> impl Iterator<Item = SyntaxError> {
        self._iter_errs(&mut Offset::default())
    }

    fn _iter_errs(&self, offset: &mut Offset) -> impl Iterator<Item = SyntaxError> {
        let mut errs = Vec::new();
        match self {
            Syntax::Operator { operands, .. } => {
                errs.extend(operands[0]._iter_errs(offset));
                errs.extend(operands[1]._iter_errs(offset));
            }
            Syntax::Brackets { brackets, inner } => {
                let bracket_kind = if let Ok(bracket) = brackets[0] {
                    Some(bracket.kind)
                } else {
                    errs.push(SyntaxError::MissingBracket(*offset));
                    None
                };
                errs.extend(inner._iter_errs(offset));
                if let Ok(bracket) = brackets[1] {
                    if let Some(kind) = bracket_kind {
                        if kind != bracket.kind {
                            errs.push(SyntaxError::MismatchedBracket(*offset, kind));
                        }
                    }
                } else {
                    errs.push(SyntaxError::MissingBracket(*offset));
                }
            }
            Syntax::Terminal(t) => {
                if let Ok(lexeme) = t {
                    *offset = lexeme.end;
                } else {
                    errs.push(SyntaxError::MissingTerminal(*offset));
                }
            }
        }
        errs.into_iter()
    }
}
