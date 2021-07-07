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
        offset: Offset,
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
            SyntaxError::MissingTerminal { offset } => (*offset, *offset),
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

// TODO Fix inner bracket err check

impl<'a> Syntax<'a, ()> {
    pub fn iter_errs(&self) -> impl Iterator<Item = SyntaxError> {
        self._iter_errs(&mut Offset::default(), &mut false)
    }

    fn _iter_errs(
        &self,
        offset: &mut Offset,
        inner_bracket_err: &mut bool,
    ) -> impl Iterator<Item = SyntaxError> {
        let mut errs = Vec::new();
        match self {
            Syntax::Operator { operator, operands } => {
                errs.extend(operands[0]._iter_errs(offset, inner_bracket_err));
                if let Some(operator) = operator {
                    *offset = operator.end;
                }
                errs.extend(operands[1]._iter_errs(offset, inner_bracket_err));
            }
            Syntax::Brackets { brackets, inner } => {
                let expected_kind = if let Ok(bracket) = brackets[0] {
                    *offset = bracket.end;
                    Some(bracket.kind.closing_counterpart())
                } else {
                    None
                };

                // Check inner first to see if this is these are innermost bracket errors.
                let mut this_inner_bracket_err = false;
                let inner_errs = inner._iter_errs(offset, &mut this_inner_bracket_err);

                if brackets[0].is_err() {
                    // Both brackets cannot be missing, we place the missing bracket error on the existing one.
                    let closing = brackets[1].as_ref().unwrap();
                    errs.push(SyntaxError::MissingBracket {
                        start: closing.start,
                        end: closing.end,
                        innermost: !this_inner_bracket_err,
                    });
                    *inner_bracket_err = true;
                }
                // We extend here to preserve correct error order.
                errs.extend(inner_errs);

                if let Ok(bracket) = brackets[1] {
                    *offset = bracket.end;
                    match expected_kind {
                        Some(kind) if kind != bracket.kind => {
                            errs.push(SyntaxError::MismatchedBracket {
                                start: bracket.start,
                                end: bracket.end,
                                innermost: !this_inner_bracket_err,
                                kind,
                            });
                            *inner_bracket_err = true;
                        }
                        _ => (),
                    }
                } else {
                    // Both brackets cannot be missing.
                    let opening = brackets[0].as_ref().unwrap();
                    errs.push(SyntaxError::MissingBracket {
                        start: opening.start,
                        end: opening.end,
                        innermost: !this_inner_bracket_err,
                    });
                    *inner_bracket_err = true;
                }
            }
            Syntax::Terminal(t) => {
                if let Ok(lexeme) = t {
                    *offset = lexeme.end;
                } else {
                    errs.push(SyntaxError::MissingTerminal { offset: *offset });
                }
            }
        }
        errs.into_iter()
    }
}
