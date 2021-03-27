use std::fmt;

use crate::{lexeme_kind::LexemeKind, offset::Offset};

#[derive(Default)]
pub struct Lexeme {
    pub string: String, // white space and unknown in front
    pub token_length: Offset,
    pub length: Offset,
    pub kind: LexemeKind,
    pub rules: [Rule; 9],
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

fn recursive_fmt(
    lexeme: &Lexeme,
    f: &mut fmt::Formatter<'_>,
    indent: u32,
    index: usize,
) -> fmt::Result {
    // Find the largest rule starting from index.
    for i in index..9 {
        let rule = &lexeme.rules[i];
        // Skip unsuccessful rules.
        if rule.length == Offset::default() {
            continue;
        }
        // Write name of rule.
        write_with_indent(f, RuleKind::from(i), indent)?;
        // Recursively print the children with a higher indent.
        recursive_fmt(lexeme, f, indent + 1, i + 1)?;
        // Tailcall to the next sibling rule with the same indentation.
        return if let Some(next) = &rule.next {
            recursive_fmt(next, f, indent, 0)
        } else {
            Ok(())
        };
    }
    // Write the current lexeme.
    write_with_indent(f, &lexeme.string, indent)?;
    // Tailcall to the next sibling lexeme with the same indentation.
    if let Some(next) = &lexeme.next {
        recursive_fmt(next, f, indent, 0)
    } else {
        Ok(())
    }
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        recursive_fmt(self, f, 0, 0)
    }
}

impl fmt::Debug for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        recursive_fmt(self, f, 0, 0)
    }
}

#[derive(Default)]
pub struct Rule {
    pub sensitive_length: Offset, // Total length including failed rules/lexemes (zero means unevaluated)
    pub length: Offset,           // Offset of next (zero means failed)
    pub next: Option<Box<Lexeme>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleKind {
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
