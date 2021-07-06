use crate::lexeme_kind::LexemeKind;
use crate::syntax::Syntax;
use std::fmt;

fn write_with_indent<T: fmt::Debug>(
    f: &mut fmt::Formatter<'_>,
    thing: T,
    indent: usize,
) -> fmt::Result {
    writeln!(f, "{:indent$}{:?}", "", thing, indent = indent * 2)
}

fn display_syntax<E>(
    f: &mut fmt::Formatter<'_>,
    syntax: &Syntax<'_, E>,
    indent: usize,
) -> fmt::Result {
    match syntax {
        Syntax::Operator { operator, operands } => {
            let kind = if let Some(operator) = operator {
                operator.kind
            } else {
                LexemeKind::App
            };
            // Prevent indents for Semicolon so that indent remains small.
            let indent = if matches!(kind, LexemeKind::Semicolon) {
                indent
            } else {
                write_with_indent(f, kind, indent)?;
                indent + 1
            };
            for operand in operands {
                display_syntax(f, operand, indent)?
            }
            Ok(())
        }
        Syntax::Brackets { inner, .. } => display_syntax(f, inner, indent),
        Syntax::Terminal(Ok(lexeme)) => write_with_indent(f, lexeme.kind, indent),
        Syntax::Terminal(Err(_)) => write_with_indent(f, None::<()>, indent),
    }
}

impl<E> fmt::Display for Syntax<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_syntax(f, self, 0)
    }
}
