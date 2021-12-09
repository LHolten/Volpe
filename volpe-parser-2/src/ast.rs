use string_interner::{StringInterner, Symbol};
use void::{ResultVoidExt, Void};

use crate::{
    lexeme_kind::LexemeKind,
    syntax::{Contained, Semicolon},
};

// this type can only hold the desugared version of the source code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Simple {
    Push(Vec<Simple>),
    Pop(Vec<usize>),
    Ident(usize),
    Raw(String),
}

pub fn replace_simple(list: &mut Vec<Simple>, name: usize, val: &[Simple]) {
    for i in (0..list.len()).rev() {
        match &mut list[i] {
            Simple::Push(inner) => replace_simple(inner, name, val),
            Simple::Pop(names) => {
                if names.contains(&name) {
                    return;
                }
            }
            Simple::Ident(n) => {
                if *n == name {
                    list.splice(i..i + 1, val.iter().cloned());
                }
            }
            _ => {}
        }
    }
}

impl Simple {
    pub fn as_ident(&self) -> usize {
        match self {
            Simple::Ident(val) => *val,
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct ASTBuilder {
    interner: StringInterner,
}

impl ASTBuilder {
    pub fn convert_contained(&mut self, syntax: &Contained<'_, Void>) -> Simple {
        match syntax {
            Contained::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => Simple::Push(self.convert_semicolon(inner)),
                LexemeKind::LCurlyBracket => {
                    let mut result = self.convert_semicolon(inner);
                    result.push(Simple::Pop(vec![0]));
                    result.insert(0, Simple::Ident(0));
                    Simple::Push(result)
                }
                LexemeKind::LSquareBracket => Simple::Pop(
                    self.convert_semicolon(inner)
                        .iter()
                        .map(Simple::as_ident)
                        .collect(),
                ),
                _ => unreachable!(),
            },
            Contained::Terminal(lexeme) => {
                let symbol = self.interner.get_or_intern(lexeme.text);
                match lexeme.kind {
                    LexemeKind::Ident => Simple::Ident(symbol.to_usize()),
                    LexemeKind::Operator => Simple::Push(vec![Simple::Ident(symbol.to_usize())]),
                    LexemeKind::Num => Simple::Raw(lexeme.text.to_string()),
                    LexemeKind::Raw => {
                        Simple::Raw(lexeme.text[2..lexeme.text.len() - 1].to_string())
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn convert_semicolon(&mut self, syntax: &Semicolon<'_, Void>) -> Vec<Simple> {
        match syntax {
            Semicolon::Semi {
                left,
                semi: operator,
                right,
            } => {
                let prev_index = left
                    .iter()
                    .rposition(|item| item.end().map(|o| o.line) == Some(operator.start.line));
                let index = prev_index.map(|i| i + 1).unwrap_or(0);

                let mut result = left
                    .iter()
                    .map(|c| self.convert_contained(c))
                    .collect::<Vec<_>>();

                result.insert(index, Simple::Push(self.convert_semicolon(right)));
                result
            }
            Semicolon::Syntax(syntax) => syntax.iter().map(|c| self.convert_contained(c)).collect(),
        }
    }
}
