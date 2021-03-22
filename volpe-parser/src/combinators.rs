use std::{cmp::max, marker::PhantomData};

use crate::{
    offset::Offset,
    syntax::Rule,
    tracker::{TError, TFunc, TInput, TResult},
};

pub struct LexemeP<const L: usize>;

impl<const L: usize> TFunc for LexemeP<L> {
    fn parse(mut t: TInput) -> TResult {
        let lexeme = t.lexeme.as_mut().unwrap();
        t.error.sensitive_length = max(t.error.sensitive_length, t.length + lexeme.length);
        if lexeme.kind.mask() & L != 0 {
            t.length += lexeme.length;
            if lexeme.next.is_none() {
                lexeme.next = Some(t.error.remaining.pop().unwrap());
            }
            t.lexeme = &mut lexeme.next;
            Ok(t)
        } else {
            Err(t.error)
        }
    }
}

pub struct RuleP<F, const R: usize> {
    f: PhantomData<F>,
}

impl<F: TFunc, const R: usize> TFunc for RuleP<F, R> {
    fn parse(mut t: TInput) -> TResult {
        let rule_ptr = &mut t.lexeme.as_mut().unwrap().rules[R] as *mut Rule;
        let rule_ptr = unsafe { &mut *rule_ptr };

        if rule_ptr.sensitive_length == Offset::default() {
            let result = F::parse(TInput {
                lexeme: t.lexeme,
                length: Offset::default(),
                error: TError {
                    sensitive_length: Offset::default(),
                    remaining: t.error.remaining,
                },
            });
            let (remaining, rule) = match result {
                Ok(input) => (
                    input.error.remaining,
                    Rule {
                        sensitive_length: input.error.sensitive_length,
                        length: input.length,
                        next: input.lexeme.take(),
                    },
                ),
                Err(error) => (
                    error.remaining,
                    Rule {
                        sensitive_length: error.sensitive_length,
                        length: Offset::default(),
                        next: None,
                    },
                ),
            };
            *rule_ptr = rule;
            t.error.remaining = remaining;
        };
        let lexeme = t.lexeme.as_mut().unwrap();
        let rule = &mut lexeme.rules[R];

        t.error.sensitive_length = max(t.error.sensitive_length, t.length + rule.sensitive_length);
        if rule.length != Offset::default() {
            t.length += rule.length;
            t.lexeme = &mut rule.next;
            Ok(t)
        } else {
            Err(t.error)
        }
    }
}

pub type Separated<F, S> = Pair<F, Many1<Pair<S, F>>>;

pub type Many1<F> = Pair<F, Many0<F>>;

pub type Opt<F> = Alt<F, Id>;

pub struct Many0<F> {
    f: PhantomData<F>,
}

impl<F: TFunc> TFunc for Many0<F> {
    fn parse(t: TInput) -> TResult {
        Alt::<Many0<F>, Id>::parse(t)
    }
}

pub struct Pair<F, G> {
    f: PhantomData<F>,
    g: PhantomData<G>,
}

impl<F: TFunc, G: TFunc> TFunc for Pair<F, G> {
    fn parse(t: TInput) -> TResult {
        G::parse(F::parse(t)?)
    }
}

pub struct Alt<F, G> {
    f: PhantomData<F>,
    g: PhantomData<G>,
}

impl<F: TFunc, G: TFunc> TFunc for Alt<F, G> {
    fn parse(t: TInput) -> TResult {
        let lexeme = t.lexeme as *mut _;
        let length = t.length;
        F::parse(t)
            .and_then(|t| {
                if t.length == length {
                    Err(t.error)
                } else {
                    Ok(t)
                }
            })
            .or_else(|error| {
                G::parse(TInput {
                    lexeme: unsafe { &mut *lexeme },
                    length,
                    error,
                })
            })
    }
}

pub struct Id;

impl TFunc for Id {
    fn parse(t: TInput) -> TResult {
        Ok(t)
    }
}
