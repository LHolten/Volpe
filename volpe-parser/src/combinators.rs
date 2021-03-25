use std::{cmp::max, marker::PhantomData};

use crate::{
    offset::Offset,
    syntax::Rule,
    tracker::{TError, TFunc, TInput, TResult},
};

pub struct LexemeP<const L: usize>;

impl<const L: usize> TFunc for LexemeP<L> {
    fn parse(mut t: TInput) -> TResult {
        for rule in &mut t.lexeme.as_mut().unwrap().rules {
            if rule.next.is_some() {
                t.error.remaining.push(rule.next.take().unwrap())
            }
        }
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
        for i in 0..R {
            let rule = &mut t.lexeme.as_mut().unwrap().rules[i];
            if rule.next.is_some() {
                t.error.remaining.push(rule.next.take().unwrap())
            }
        }
        let rule = &mut t.lexeme.as_mut().unwrap().rules[R] as *mut Rule;
        if t.lexeme.as_mut().unwrap().rules[R].sensitive_length == Offset::default() {
            // current rule is not tried yet
            let result = F::parse(TInput {
                lexeme: t.lexeme,
                length: Offset::default(),
                error: TError {
                    sensitive_length: Offset::default(),
                    remaining: t.error.remaining,
                },
            });
            let mut empty = None;
            let tracker = match result {
                Ok(input) => input,
                Err(error) => TInput {
                    lexeme: &mut empty, // this will become None in the rule
                    length: Offset::default(),
                    error,
                },
            };
            let rule = unsafe { &mut *rule };
            assert!(rule.next.is_none());
            *rule = Rule {
                sensitive_length: tracker.error.sensitive_length,
                length: tracker.length,
                next: tracker.lexeme.take(),
            };
            t.error.remaining = tracker.error.remaining;
        };
        let rule = unsafe { &mut *rule };

        t.error.sensitive_length = max(t.error.sensitive_length, t.length + rule.sensitive_length);
        if rule.length != Offset::default() {
            t.length += rule.length;
            if rule.next.is_none() {
                rule.next = Some(t.error.remaining.pop().unwrap());
            }
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
        Opt::<Pair<NotOpt<F>, Many0<F>>>::parse(t)
    }
}

pub struct NotOpt<F> {
    f: PhantomData<F>,
}

impl<F: TFunc> TFunc for NotOpt<F> {
    fn parse(t: TInput) -> Result<TInput, TError> {
        let length = t.length;
        F::parse(t).and_then(|t| {
            if t.length == length {
                Err(t.error)
            } else {
                Ok(t)
            }
        })
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
        F::parse(t).or_else(|error| {
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
