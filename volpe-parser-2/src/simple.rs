use std::mem::take;

use void::{ResultVoidExt, Void};

use crate::{
    lexeme_kind::LexemeKind,
    offset::Range,
    syntax::{Contained, Semicolon},
};

// this type can only hold the desugared version of the source code
#[derive(Clone)]
pub enum Simple<'a> {
    Push(Vec<Simple<'a>>),
    Pop(Range<'a>),
    Ident(Range<'a>),
    Raw(Range<'a>),
}

impl<'a> Contained<'a, Void> {
    pub fn convert(&self) -> Vec<Simple<'a>> {
        match self {
            Contained::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => vec![Simple::Push(inner.convert())],
                LexemeKind::LCurlyBracket => vec![Simple::Push(inner.convert())],
                LexemeKind::LSquareBracket => match inner {
                    Semicolon::Semi { .. } => unreachable!(),
                    Semicolon::Syntax(list) => list
                        .iter()
                        .map(|c| match c {
                            Contained::Brackets { .. } => panic!(),
                            Contained::Terminal(lexeme) => Simple::Pop(lexeme.range),
                        })
                        .rev()
                        .collect(),
                },
                _ => unreachable!(),
            },
            Contained::Terminal(lexeme) => match lexeme.kind {
                LexemeKind::Ident => vec![Simple::Ident(lexeme.range)],
                LexemeKind::Operator => vec![Simple::Ident(lexeme.range)],
                LexemeKind::Num => vec![Simple::Raw(lexeme.range)],
                LexemeKind::Raw => vec![Simple::Raw(lexeme.range.raw_inner())],
                _ => unreachable!(),
            },
        }
    }
}

impl<'a> Semicolon<'a, Void> {
    pub fn convert(&self) -> Vec<Simple<'a>> {
        let mut out = vec![];
        let mut line_args = vec![];
        let line = match self {
            Semicolon::Semi { left, right, .. } => {
                out = right.convert();
                left
            }
            Semicolon::Syntax(line) => {
                out.push(Simple::Ident(Default::default()));
                line_args.push(Simple::Pop(Default::default()));
                line
            }
        };
        for ast in line {
            let simple = ast.convert();
            for s in simple {
                if matches!(s, Simple::Ident(_)) {
                    line_args.push(Simple::Push(take(&mut out)))
                }
                out.push(s)
            }
        }
        out.extend(line_args.into_iter().rev());
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::{file::File, offset::Offset};

    fn print_simple(input: &str) {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), input.to_string())
            .unwrap();
        let syntax = file.rule().collect().unwrap();
        println!("{:?}", syntax.convert());
    }

    #[test]
    fn curly_semi() {
        print_simple("(expr(v)[v])");
        print_simple("{expr;}");
    }
}
