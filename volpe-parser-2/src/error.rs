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
    MissingTerminal {
        start: Offset,
        end: Offset,
    },
    MissingBracket {
        start: Offset,
        end: Offset,
        innermost: bool,
    },
    MismatchedBracket {
        start: Offset,
        end: Offset,
        innermost: bool,
        kind: LexemeKind,
    },
    // MisplacedOperator { offset: Offset }
}

impl SyntaxError {
    pub fn get_range(&self) -> (Offset, Offset) {
        match self {
            SyntaxError::MissingTerminal { start, end, .. } => (*start, *end),
            SyntaxError::MissingBracket { start, end, .. } => (*start, *end),
            SyntaxError::MismatchedBracket { start, end, .. } => (*start, *end),
        }
    }

    pub fn should_show(&self) -> bool {
        match self {
            SyntaxError::MissingTerminal { .. } => true,
            SyntaxError::MissingBracket { innermost, .. } => *innermost,
            SyntaxError::MismatchedBracket { innermost, .. } => *innermost,
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::MissingTerminal { .. } => write!(f, "Missing terminal"),
            SyntaxError::MissingBracket { .. } => write!(f, "Missing bracket"),
            SyntaxError::MismatchedBracket { kind, .. } => {
                write!(f, "Mismatched bracket kind, expected {:?}", kind)
            }
        }
    }
}

impl Error for SyntaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl<'a, E: std::fmt::Debug> Syntax<'a, E> {
    pub fn iter_errs(&self) -> impl Iterator<Item = SyntaxError> {
        self._iter_errs(&mut false)
    }

    fn _iter_errs(&self, inner_bracket_err: &mut bool) -> impl Iterator<Item = SyntaxError> {
        let mut errs = Vec::new();
        match self {
            Syntax::Operator { operator, operands } => {
                for operand in operands {
                    if let Syntax::Terminal(Err(_)) = operand.as_ref() {
                        let operator = operator.as_ref().unwrap();
                        errs.push(SyntaxError::MissingTerminal {
                            start: operator.start,
                            end: operator.end,
                        });
                    } else {
                        errs.extend(operand._iter_errs(inner_bracket_err));
                    }
                }
            }
            Syntax::Brackets {
                brackets: [opening, closing],
                inner,
            } => {
                let mut this_inner_bracket_err = false;
                if let Some(inner) = inner {
                    errs.extend(inner._iter_errs(&mut this_inner_bracket_err));
                }

                if let Ok(opening) = opening {
                    if let Ok(closing) = closing {
                        // Handle mismatched brackets.
                        if opening.kind.bracket_counterpart() != closing.kind {
                            errs.push(SyntaxError::MismatchedBracket {
                                start: opening.start,
                                end: opening.end,
                                innermost: !this_inner_bracket_err,
                                kind: opening.kind.bracket_counterpart(),
                            });
                            errs.push(SyntaxError::MismatchedBracket {
                                start: closing.start,
                                end: closing.end,
                                innermost: !this_inner_bracket_err,
                                kind: closing.kind.bracket_counterpart(),
                            });
                            *inner_bracket_err = true;
                        }
                    } else {
                        errs.push(SyntaxError::MissingBracket {
                            start: opening.start,
                            end: opening.end,
                            innermost: !this_inner_bracket_err,
                        });
                        *inner_bracket_err = true;
                    }
                } else {
                    // Both brackets cannot be missing.
                    let closing = closing.as_ref().unwrap();
                    errs.push(SyntaxError::MissingBracket {
                        start: closing.start,
                        end: closing.end,
                        innermost: !this_inner_bracket_err,
                    });
                    *inner_bracket_err = true;
                }
            }
            Syntax::Terminal(Ok(_)) => (),
            // We can only get here if there is no other code - empty file.
            Syntax::Terminal(Err(_)) => errs.push(SyntaxError::MissingTerminal {
                start: Offset::default(),
                end: Offset::default(),
            }),
        }
        errs.into_iter()
    }
}
