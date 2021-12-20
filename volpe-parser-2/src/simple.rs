use void::{ResultVoidExt, Void};

use crate::{
    lexeme_kind::LexemeKind,
    offset::Range,
    syntax::{Contained, Semicolon},
};

// this type can only hold the desugared version of the source code
#[derive(Debug, Clone)]
pub enum Simple<'a> {
    Push(Box<Simple<'a>>),
    Scope(Vec<Simple<'a>>),
    Pop(Range<'a>),
    Ident(Range<'a>),
    Raw(Range<'a>),
}

impl<'a> Simple<'a> {
    fn as_ident(&self) -> Range<'a> {
        match self {
            Simple::Ident(range) => *range,
            _ => todo!(),
        }
    }
}

impl<'a> Contained<'a, Void> {
    pub fn convert(&self) -> Vec<Simple<'a>> {
        match self {
            Contained::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => {
                    vec![Simple::Push(Simple::Scope(inner.convert()).into())]
                }
                LexemeKind::LCurlyBracket => {
                    let mut result = inner.convert();
                    result.push(Simple::Pop(Default::default()));
                    result.insert(0, Simple::Ident(Default::default()));
                    vec![Simple::Scope(result)]
                }
                LexemeKind::LSquareBracket => inner
                    .convert()
                    .iter()
                    .map(Simple::as_ident)
                    .map(Simple::Pop)
                    .rev()
                    .collect(),
                _ => unreachable!(),
            },
            Contained::Terminal(lexeme) => match lexeme.kind {
                LexemeKind::Ident => vec![Simple::Ident(lexeme.range)],
                LexemeKind::Operator => vec![Simple::Push(Simple::Ident(lexeme.range).into())],
                LexemeKind::Num => vec![Simple::Raw(lexeme.range)],
                LexemeKind::Raw => vec![Simple::Raw(lexeme.range.raw_inner())],
                _ => unreachable!(),
            },
        }
    }
}

impl<'a> Semicolon<'a, Void> {
    pub fn convert(&self) -> Vec<Simple<'a>> {
        match self {
            Semicolon::Semi {
                left,
                semi: operator,
                right,
            } => {
                let prev_index = left.iter().rposition(|item| {
                    item.end().map(|o| o.line) == Some(operator.range.start.line)
                });
                let index = prev_index.map(|i| i + 1).unwrap_or(0);

                let mut result = left.iter().flat_map(Contained::convert).collect::<Vec<_>>();

                result.insert(index, Simple::Push(Simple::Scope(right.convert()).into()));
                result
            }
            Semicolon::Syntax(syntax) => syntax.iter().flat_map(Contained::convert).collect(),
        }
    }
}
