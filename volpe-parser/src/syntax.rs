use std::fmt::Debug;

use crate::{lexeme_kind::LexemeKind, offset::Offset};

#[derive(Default)]
pub struct Lexeme {
    pub string: String, // white space and unknown in front
    pub length: Offset,
    pub kind: LexemeKind,
    pub rules: [Rule; 9],
    pub next: Option<Box<Lexeme>>,
}

const INDENT: &'static str = "  ";

impl Debug for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        let mut indent = 0;
        let mut next_lexemes = vec![self];

        while let Some(lexeme) = next_lexemes.pop() {
            for (i, rule) in lexeme.rules.iter().enumerate() {
                if rule.length > Offset::default() {
                    for _ in 0..indent { output.push_str(INDENT) }
                    output.push_str(&format!("{:?}\n", RuleKind::from(i)));
                    if let Some(next) = &rule.next {
                        next_lexemes.push(next)
                    }
                    indent += 1;
                }
            }
            for _ in 0..indent { output.push_str(INDENT) }
            output.push_str(&format!("{:?}\n", lexeme.string));
            if let Some(next) = &lexeme.next {
                next_lexemes.push(next)
            } else {
                indent -= 1;
            }
        }
        
        f.write_str(&output)
    }
}

#[derive(Default)]
pub struct Rule {
    pub sensitive_length: Offset, // total length including failed rules/lexemes (zero means unevaluated)
    pub length: Offset,           // offset of next (zero means failed)
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
    File,
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
