use void::{ResultVoidExt, Void};

use crate::{
    lexeme_kind::LexemeKind,
    offset::Range,
    syntax::{Contained, Semicolon},
};

// this type can only hold the desugared version of the source code
#[derive(Debug, Clone)]
pub enum Simple<'a> {
    Push(Vec<Simple<'a>>),
    Pop(Vec<Range<'a>>),
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
    pub fn convert(&self) -> Simple<'a> {
        match self {
            Contained::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => Simple::Push(inner.convert()),
                LexemeKind::LCurlyBracket => {
                    let mut result = inner.convert();
                    result.push(Simple::Pop(vec![Default::default()]));
                    result.insert(0, Simple::Ident(Default::default()));
                    Simple::Push(result)
                }
                LexemeKind::LSquareBracket => {
                    Simple::Pop(inner.convert().iter().map(Simple::as_ident).collect())
                }
                _ => unreachable!(),
            },
            Contained::Terminal(lexeme) => match lexeme.kind {
                LexemeKind::Ident => Simple::Ident(lexeme.range),
                LexemeKind::Operator => Simple::Push(vec![Simple::Ident(lexeme.range)]),
                LexemeKind::Num => Simple::Raw(lexeme.range),
                LexemeKind::Raw => Simple::Raw(lexeme.range.raw_inner()),
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

                let mut result = left.iter().map(Contained::convert).collect::<Vec<_>>();

                result.insert(index, Simple::Push(right.convert()));
                result
            }
            Semicolon::Syntax(syntax) => syntax.iter().map(Contained::convert).collect(),
        }
    }
}
