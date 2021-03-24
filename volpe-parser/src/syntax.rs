use std::fmt;

use crate::{lexeme_kind::LexemeKind, offset::Offset};

#[derive(Default)]
pub struct Lexeme {
    pub string: String, // white space and unknown in front
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

fn recursive_fmt(this: &Lexeme, f: &mut fmt::Formatter<'_>, mut indent: u32) -> fmt::Result {
    // Rules are sorted so that the largest ones are first.
    // This means we can use next_lexemes like a stack where
    // the last value corresponds to the `.next` of the smallest rule.
    // These are used when we reach the end of a lexeme chain.
    let mut next_lexemes = Vec::new();

    // Iterate over all rules that start at this lexeme.
    for (i, rule) in this.rules.iter().enumerate() {
        // Skip unsuccessful rules.
        if rule.length <= Offset::default() {
            continue;
        }
        // Write name of rule.
        write_with_indent(f, RuleKind::from(i), indent)?;
        indent += 1;
        // Remember which lexeme is next after this rule.
        if let Some(next) = &rule.next {
            next_lexemes.push(next)
        }
    }
    // Write the current lexeme.
    write_with_indent(f, &this.string, indent)?;
    // Repeat whole process for the next lexeme in the chain.
    if let Some(next) = &this.next {
        recursive_fmt(next, f, indent)?;
    }
    // We reached the end of the chain.
    // Now we have to use the remembered `next` lexemes.
    while let Some(lexeme) = next_lexemes.pop() {
        indent -= 1;
        recursive_fmt(lexeme, f, indent)?;
    }
    Ok(())
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        recursive_fmt(self, f, 0)
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
