use std::{fmt, usize};

use crate::{lexeme_kind::LexemeKind, offset::Offset};

pub const RULE_COUNT: usize = 10;

#[derive(Default, PartialEq, Debug)]
pub struct Lexeme {
    pub string: String, // white space and unknown at the end
    pub token_length: Offset,
    pub length: Offset,
    pub kind: LexemeKind,
    pub rules: [Rule; RULE_COUNT],
    pub next: Option<Box<Lexeme>>,
}

impl Lexeme {
    fn next_kind(&self, rule_from: usize) -> Option<RuleKind> {
        for rule_kind in rule_from..RULE_COUNT {
            let rule = &self.rules[rule_kind];
            if rule.length != Offset::default() {
                return Some(RuleKind::from(rule_kind));
            }
        }
        None
    }

    fn first_syntax(&self) -> Syntax<'_> {
        Syntax {
            lexeme: self,
            kind: self.next_kind(0),
        }
    }

    pub fn get_text(&self) -> String {
        let mut text = String::new();
        fn rec(syntax: Syntax, text: &mut String) {
            if syntax.kind.is_none() {
                text.push_str(&syntax.lexeme.string);
                return;
            }
            for child in syntax {
                rec(child, text);
            }
        }
        for syntax in self {
            rec(syntax, &mut text);
        }
        text
    }

    pub fn get_size(&self) -> Offset {
        let mut size = Offset::default();
        for syntax in self {
            size += match syntax.kind {
                Some(kind) => syntax.lexeme.rules[kind as usize].length,
                None => syntax.lexeme.length,
            };
        }
        size
    }
}

impl<'a> IntoIterator for &'a Lexeme {
    type Item = Syntax<'a>;

    type IntoIter = SyntaxIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SyntaxIter(Some(self.first_syntax()))
    }
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl fmt::Debug for Syntax<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rule_kind) = self.kind {
            write!(f, "{:?} ", rule_kind)?;
            f.debug_list().entries(*self).finish()
        } else {
            write!(f, "{:?}: {:?}", self.lexeme.kind, self.lexeme.string)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Syntax<'a> {
    pub lexeme: &'a Lexeme,
    pub kind: Option<RuleKind>, // None means that this is a lexeme and not a rule
}

impl Syntax<'_> {
    pub fn rule_length(&self) -> Option<Offset> {
        self.kind
            .map(|kind| self.lexeme.rules[kind as usize].length)
    }
}

impl<'a> IntoIterator for Syntax<'a> {
    type Item = Syntax<'a>;

    type IntoIter = SyntaxIter<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        SyntaxIter(self.kind.map(|rule_kind| {
            self.kind = self.lexeme.next_kind(rule_kind as usize + 1);
            self
        }))
    }
}

#[derive(Clone, Copy)]
pub struct SyntaxIter<'a>(Option<Syntax<'a>>);

impl<'a> Iterator for SyntaxIter<'a> {
    type Item = Syntax<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|inner| {
            let next = if let Some(rule_kind) = inner.kind {
                &inner.lexeme.rules[rule_kind as usize].next
            } else {
                &inner.lexeme.next
            };
            self.0 = next.as_ref().map(|lexeme| lexeme.first_syntax());
            inner
        })
    }
}

#[derive(Default, PartialEq, Debug)]
pub struct Rule {
    pub sensitive_length: Offset, // Total length including failed rules/lexemes (zero means unevaluated)
    pub length: Offset,           // Offset of next (zero means failed)
    pub next: Option<Box<Lexeme>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleKind {
    Expr,
    App,
    Func,
    Or,
    And,
    Op1,
    Op2,
    Op3,
    Block,
    Tuple,
}

impl From<usize> for RuleKind {
    fn from(val: usize) -> Self {
        match val {
            v if v == Self::Expr as usize => Self::Expr,
            v if v == Self::App as usize => Self::App,
            v if v == Self::Func as usize => Self::Func,
            v if v == Self::Or as usize => Self::Or,
            v if v == Self::And as usize => Self::And,
            v if v == Self::Op1 as usize => Self::Op1,
            v if v == Self::Op2 as usize => Self::Op2,
            v if v == Self::Op3 as usize => Self::Op3,
            v if v == Self::Block as usize => Self::Block,
            v if v == Self::Tuple as usize => Self::Tuple,
            _ => unreachable!(),
        }
    }
}

pub trait OrRemaining {
    #[allow(clippy::vec_box)]
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
