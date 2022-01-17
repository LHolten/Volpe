use void::Void;

use crate::{error::SyntaxError, syntax::Contained};

impl<'a> Contained<'a, ()> {
    pub fn collect(self) -> Result<Contained<'a, Void>, Vec<SyntaxError<'a>>> {
        match self {
            Contained::Brackets { brackets, inner } => Ok(Contained::Brackets {
                inner: collect_semi(inner)?,
                brackets: match brackets {
                    [Err(()), Ok(lex)] => Err(vec![SyntaxError::UnmatchedBracket(lex)]),
                    [Ok(lex), Err(())] => Err(vec![SyntaxError::UnmatchedBracket(lex)]),
                    [Ok(left), Ok(right)] => {
                        if left.kind.bracket_counterpart() == right.kind {
                            Ok([Ok(left), Ok(right)])
                        } else {
                            Err(vec![
                                SyntaxError::MismatchedBracket(left),
                                SyntaxError::MismatchedBracket(right),
                            ])
                        }
                    }
                    [Err(()), Err(())] => unreachable!(),
                }?,
            }),
            Contained::Terminal(t) => Ok(Contained::Terminal(t)),
        }
    }
}

pub fn collect_semi(
    list: Vec<Vec<Contained<'_, ()>>>,
) -> Result<Vec<Vec<Contained<'_, Void>>>, Vec<SyntaxError<'_>>> {
    let (mut result, mut errors) = (vec![], vec![]);
    list.into_iter().for_each(|r| {
        let mut new = vec![];
        r.into_iter().for_each(|r| match r.collect() {
            Ok(b) => new.push(b),
            Err(e) => errors.extend(e),
        });
        result.push(new)
    });
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(result)
}
