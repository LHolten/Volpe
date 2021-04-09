use std::{fmt, usize};

use crate::{lexeme_kind::LexemeKind, offset::Offset};

#[derive(Default)]
pub struct Lexeme {
    pub string: String, // white space and unknown at the end
    pub token_length: Offset,
    pub length: Offset,
    pub kind: LexemeKind,
    pub rules: [Rule; 10],
    pub next: Option<Box<Lexeme>>,
}

impl Lexeme {
    fn next_kind(&self, rule_from: usize) -> Option<RuleKind> {
        for rule_kind in rule_from..10 {
            let rule = &self.rules[rule_kind];
            if rule.length != Offset::default() {
                return Some(RuleKind::from(rule_kind));
            }
        }
        None
    }
}

impl fmt::Display for Box<Lexeme> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl fmt::Debug for Box<Lexeme> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = SyntaxIter(Some(self.into()));
        f.debug_list().entries(iter).finish()
    }
}

impl fmt::Debug for Syntax<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rule_kind) = self.kind {
            write!(f, "{:?} ", rule_kind)?;
            f.debug_list().entries(*self).finish()?
        } else {
            write!(f, "{:?}: {:?}", self.lexeme.kind, self.lexeme.string)?
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Syntax<'a> {
    lexeme: &'a Box<Lexeme>,
    kind: Option<RuleKind>, // None means that this is a lexeme and not a rule
}

impl<'a> From<&'a Box<Lexeme>> for Syntax<'a> {
    fn from(lexeme: &'a Box<Lexeme>) -> Self {
        Self {
            lexeme,
            kind: lexeme.next_kind(0),
        }
    }
}

impl<'a> IntoIterator for Syntax<'a> {
    type Item = Syntax<'a>;

    type IntoIter = SyntaxIter<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        if let Some(rule_kind) = self.kind {
            self.kind = self.lexeme.next_kind(rule_kind as usize + 1);
            SyntaxIter(Some(self))
        } else {
            SyntaxIter(None)
        }
    }
}

#[derive(Clone, Copy)]
pub struct SyntaxIter<'a>(Option<Syntax<'a>>);

impl<'a> Iterator for SyntaxIter<'a> {
    type Item = Syntax<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner) = self.0 {
            let next = if let Some(rule_kind) = inner.kind {
                &inner.lexeme.rules[rule_kind as usize].next
            } else {
                &inner.lexeme.next
            };
            self.0 = next.as_ref().map(Syntax::from);
            Some(inner)
        } else {
            None
        }
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
    Tuple,
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
            v if v == Self::Tuple as usize => Self::Tuple,
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
