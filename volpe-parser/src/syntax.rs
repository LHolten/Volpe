use std::{
    cell::Cell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use crate::{internal::UpgradeInternal, lexeme_kind::LexemeKind, offset::Offset};

#[derive(Clone)]
pub enum Syntax {
    Lexeme(Rc<Lexeme>),
    Rule(Rc<Rule>),
}

impl Default for Syntax {
    fn default() -> Self {
        Self::Lexeme(Default::default())
    }
}

impl Debug for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Syntax::Lexeme(pos) => pos.string.fmt(f),
            Syntax::Rule(rule) => {
                rule.kind.fmt(f)?;
                rule.children
                    .iter()
                    .filter(|s| Syntax::is_success(*s))
                    .collect::<Vec<&Syntax>>()
                    .fmt(f)
            }
        }
    }
}

impl Syntax {
    pub fn is_success(&self) -> bool {
        match self {
            Syntax::Lexeme(_) => true,
            Syntax::Rule(rule) => rule.next.upgrade().is_some(),
        }
    }

    pub fn len(&self) -> (Offset, Offset) {
        match self {
            Syntax::Lexeme(lexeme) => (lexeme.length, lexeme.length),
            Syntax::Rule(rule) => (rule.offset, rule.length),
        }
    }

    pub fn next_lexeme(&self) -> Rc<Lexeme> {
        match self {
            Syntax::Lexeme(lexeme) => lexeme.next.upgrade().unwrap(),
            Syntax::Rule(rule) => rule.next.upgrade().unwrap(),
        }
    }

    pub fn first_lexeme(&self) -> Rc<Lexeme> {
        match self {
            Syntax::Lexeme(lexeme) => lexeme.clone(),
            Syntax::Rule(rule) => {
                for child in &rule.children {
                    if child.len().0 != Offset::default() {
                        return child.first_lexeme();
                    }
                }
                unreachable!()
            }
        }
    }

    pub fn text(&self) -> String {
        format!("{:?}", self.first_lexeme())
    }
}

#[derive(Default)]
pub struct Lexeme {
    pub string: String, // white space and unknown in front
    pub length: Offset,
    pub kind: LexemeKind,
    pub rules: [Cell<Weak<Rule>>; 9],
    pub next: Cell<Weak<Lexeme>>,
}

impl Debug for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string)?;
        if let Some(next) = self.next.upgrade() {
            next.fmt(f)
        } else {
            Ok(())
        }
    }
}

pub struct Rule {
    pub children: Vec<Syntax>,
    pub length: Offset, // total length including failed rules/lexems
    pub kind: RuleKind,
    pub offset: Offset,     // offset of next
    pub next: Weak<Lexeme>, // is Some when rule succeeded
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
