use std::ops::ControlFlow;

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
    Num(i32),
    Raw(String),
}

impl Simple {
    pub fn as_ident(&self) -> usize {
        match self {
            Simple::Ident(val) => *val,
            _ => unreachable!(),
        }
    }

    pub fn replace(&mut self, name: usize, val: &Simple) -> ControlFlow<()> {
        match self {
            Simple::Push(inner) => {
                inner
                    .iter_mut()
                    .rev()
                    .try_for_each(|item| item.replace(name, val));
                ControlFlow::Continue(())
            }
            Simple::Pop(names) => {
                if names.contains(&name) {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
            Simple::Ident(id) => {
                if *id == name {
                    *self = val.clone()
                }
                ControlFlow::Continue(())
            }
            _ => ControlFlow::Continue(()),
        }
    }
}

#[derive(Default)]
pub struct ASTBuilder {
    interner: StringInterner,
}

impl ASTBuilder {
    pub fn convert_contained<'a>(&mut self, syntax: &Contained<'a, Void>) -> Simple {
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
                    LexemeKind::Num => Simple::Num(lexeme.text.parse().unwrap()),
                    LexemeKind::Wasm => Simple::Raw(lexeme.text.to_string()),
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn convert_semicolon<'a>(
        &mut self,
        syntax: impl AsRef<Semicolon<'a, Void>>,
    ) -> Vec<Simple> {
        match syntax.as_ref() {
            Semicolon::Semi {
                left,
                semi: operator,
                right,
            } => {
                let prev_index = left
                    .iter()
                    .rposition(|item| item.end() == Some(operator.start));
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
