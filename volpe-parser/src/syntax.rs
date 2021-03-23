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

const INDENT: &'static str = "  ";

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut indent = 0;
        // we treat next_lexemes as a stack of remembered lexemes
        let mut next_lexemes = vec![self];

        while let Some(lexeme) = next_lexemes.pop() {
            // for each rule we check if it is valid by looking at length
            // if valid, we write the rule and remember the next lexeme
            // once we finish a lexeme chain we use these remembered lexemes to continue
            for (i, rule) in lexeme.rules.iter().enumerate() {
                if rule.length > Offset::default() {
                    // write the rule
                    for _ in 0..indent {
                        f.write_str(INDENT)?;
                    }
                    write!(f, "{:?}\n", RuleKind::from(i))?;
                    // remember the next lexeme for later
                    if let Some(next) = &rule.next {
                        next_lexemes.push(next)
                    }
                    indent += 1;
                }
            }
            // write the lexeme
            for _ in 0..indent {
                f.write_str(INDENT)?;
            }
            write!(f, "{:?}\n", lexeme.string)?;
            // this is where we follow a lexeme chain
            if let Some(next) = &lexeme.next {
                next_lexemes.push(next)
            } else {
                // we reached the end of a chain, i.e. end of a rule
                indent -= 1;
            }
        }
        Ok(())
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
