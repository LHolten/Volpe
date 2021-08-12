use crate::lexeme_kind::LexemeKind;
use crate::syntax::Syntax;
use std::fmt;

fn with_indent<T: fmt::Debug>(f: &mut fmt::Formatter, thing: T, indent: usize) -> fmt::Result {
    writeln!(f, "{:i$}{:?}", "", thing, i = indent * 2)
}

fn with_indent2<T: fmt::Debug, G: fmt::Debug>(
    f: &mut fmt::Formatter,
    first: T,
    second: G,
    indent: usize,
) -> fmt::Result {
    writeln!(f, "{:i$}{:?} {:?}", "", first, second, i = indent * 2)
}

fn display_syntax<E>(
    f: &mut fmt::Formatter<'_>,
    syntax: &Syntax<'_, E>,
    indent: usize,
    hide_semicolon: bool,
) -> fmt::Result {
    match syntax {
        Syntax::Operator { operator, operands } => {
            let kind = if let Some(operator) = operator {
                operator.kind
            } else {
                LexemeKind::App
            };
            // Prevent indents for Semicolon so that indent remains small.
            let indent = if hide_semicolon && matches!(kind, LexemeKind::Semicolon) {
                indent
            } else {
                with_indent(f, kind, indent)?;
                indent + 1
            };
            for operand in operands {
                display_syntax(f, operand, indent, hide_semicolon)?
            }
            Ok(())
        }
        Syntax::Brackets { inner, .. } => inner
            .as_ref()
            .map(|inner| display_syntax(f, inner, indent, hide_semicolon))
            .unwrap_or(Ok(())),
        Syntax::Terminal(Ok(lexeme)) => with_indent2(f, lexeme.kind, lexeme.text, indent),
        Syntax::Terminal(Err(_)) => with_indent(f, None::<()>, indent),
    }
}

impl<E> fmt::Display for Syntax<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_syntax(f, self, 0, f.alternate())
    }
}
