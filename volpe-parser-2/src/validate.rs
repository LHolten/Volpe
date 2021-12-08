use void::Void;

use crate::{
    error::SyntaxError,
    syntax::{Contained, Semicolon},
};

impl<'a> Contained<'a, ()> {
    pub fn collect(self) -> Result<Contained<'a, Void>, Vec<SyntaxError<'a>>> {
        match self {
            Contained::Brackets { brackets, inner } => Ok(Contained::Brackets {
                inner: inner.collect()?.into(),
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

fn collect_errors(
    list: Vec<Contained<'_, ()>>,
) -> Result<Vec<Contained<'_, Void>>, Vec<SyntaxError<'_>>> {
    let (mut result, mut errors) = (vec![], vec![]);
    list.into_iter().for_each(|r| match r.collect() {
        Ok(b) => result.push(b),
        Err(e) => errors.extend(e),
    });
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(result)
}

impl<'a> Semicolon<'a, ()> {
    pub fn collect(self) -> Result<Semicolon<'a, Void>, Vec<SyntaxError<'a>>> {
        match self {
            Semicolon::Semi { left, semi, right } => {
                match collect_errors(left) {
                    Err(mut errors) => {
                        // check if there are more errors in the right hand side
                        errors.extend(right.collect().err().into_iter().flatten());
                        Result::Err(errors)
                    }
                    Ok(left) => Ok(Semicolon::Semi {
                        left,
                        semi,
                        right: right.collect()?.into(),
                    }),
                }
            }
            Semicolon::Syntax(syntax) => Ok(Semicolon::Syntax(collect_errors(syntax)?)),
        }
    }
}
