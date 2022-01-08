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
    Pop(Range<'a>),
    Ident(Range<'a>),
    Raw(Range<'a>),
}

impl<'a> Simple<'a> {
    fn as_ident(&self) -> Range<'a> {
        match self {
            Simple::Ident(range) => *range,
            _ => todo!("{:?}", self),
        }
    }
}

impl<'a> Contained<'a, Void> {
    pub fn convert(&self) -> Vec<Simple<'a>> {
        match self {
            Contained::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => {
                    vec![Simple::Push(inner.convert(None))]
                }
                LexemeKind::LCurlyBracket => {
                    let mut result = inner.convert(Some(Simple::Ident(Default::default())));
                    result.push(Simple::Pop(Default::default()));
                    vec![Simple::Push(result)]
                }
                LexemeKind::LSquareBracket => inner
                    .convert(None)
                    .iter()
                    .map(Simple::as_ident)
                    .map(Simple::Pop)
                    .rev()
                    .collect(),
                _ => unreachable!(),
            },
            Contained::Terminal(lexeme) => match lexeme.kind {
                LexemeKind::Ident => vec![Simple::Ident(lexeme.range)],
                LexemeKind::Operator => vec![Simple::Ident(lexeme.range)],
                LexemeKind::Num => vec![Simple::Push(vec![Simple::Raw(lexeme.range)])],
                LexemeKind::Raw => vec![Simple::Raw(lexeme.range.raw_inner())],
                _ => unreachable!(),
            },
        }
    }
}

impl<'a> Semicolon<'a, Void> {
    pub fn convert(&self, end: Option<Simple<'a>>) -> Vec<Simple<'a>> {
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

                result.insert(index, Simple::Push(right.convert(end)));
                result
            }
            Semicolon::Syntax(syntax) => {
                let mut res = end.into_iter().collect::<Vec<_>>();
                res.extend(syntax.iter().flat_map(Contained::convert));
                res
            }
        }
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
        println!("{:?}", syntax.convert(None));
    }

    #[test]
    fn curly_semi() {
        print_simple("(expr(v)[v])");
        print_simple("{expr;}");
    }
}
