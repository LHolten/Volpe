use std::fmt;

use crate::{lexeme_kind::LexemeKind, offset::Offset};

#[derive(Default)]
pub struct Lexeme {
    pub string: String, // white space and unknown in front
    pub token_length: Offset,
    pub length: Offset,
    pub kind: LexemeKind,
    pub rules: [Rule; 10],
    pub next: Option<Box<Lexeme>>,
}

fn write_with_indent<D: fmt::Debug>(
    f: &mut fmt::Formatter<'_>,
    thing: D,
    indent: u32,
) -> fmt::Result {
    for _ in 0..indent {
        f.write_str("  ")?;
    }
    write!(f, "{:?}\n", thing)
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use next_lexemes as a stack for lexemes and indentation.
        let mut next_lexemes = vec![(self, 0)];
        while let Some((lexeme, mut indent)) = next_lexemes.pop() {
            for (i, rule) in lexeme.rules.iter().enumerate() {
                // Skip unsuccessful rules.
                if rule.length == Offset::default() {
                    continue;
                }
                // Write name of rule.
                write_with_indent(f, RuleKind::from(i), indent)?;
                // Save the next lexeme and indentation level.
                if let Some(next) = &rule.next {
                    next_lexemes.push((next, indent));
                }
                // Increase the indentation for this scope.
                indent += 1;
            }
            // Write the current lexeme.
            write_with_indent(f, (lexeme.kind, &lexeme.string), indent)?;
            // Add the next lexeme so the whole process can be repeated for it.
            if let Some(next) = &lexeme.next {
                next_lexemes.push((next, indent));
            }
        }
        Ok(())
    }
}

// TODO Implement fmt::Debug for Lexeme that shows unsuccessful rules too.

#[derive(Default)]
pub struct Rule {
    pub sensitive_length: Offset, // Total length including failed rules/lexemes (zero means unevaluated)
    pub length: Offset,           // Offset of next (zero means failed)
    pub next: Option<Box<Lexeme>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleKind {
    Tuple,
    Expr,
    Stmt,
    App,
    Func,
    Or,
    And,
    Op1,
    Op2,
    Op3,
}

impl From<usize> for RuleKind {
    fn from(val: usize) -> Self {
        match val {
            v if v == Self::Tuple as usize => Self::Tuple,
            v if v == Self::Expr as usize => Self::Expr,
            v if v == Self::Stmt as usize => Self::Stmt,
            v if v == Self::App as usize => Self::App,
            v if v == Self::Func as usize => Self::Func,
            v if v == Self::Or as usize => Self::Or,
            v if v == Self::And as usize => Self::And,
            v if v == Self::Op1 as usize => Self::Op1,
            v if v == Self::Op2 as usize => Self::Op2,
            v if v == Self::Op3 as usize => Self::Op3,
            _ => unreachable!(),
        }
    }
}

pub trait OrRemaining {
    fn or_remaining(&mut self, remaining: &mut Vec<Box<Lexeme>>) -> &mut Self;
}

impl OrRemaining for Option<Box<Lexeme>> {
    fn or_remaining(&mut self, remaining: &mut Vec<Box<Lexeme>>) -> &mut Self {
        if self.is_none() {
            *self = remaining.pop()
        }
        self
    }
}
