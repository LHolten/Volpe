use std::{cmp::max, marker::PhantomData, mem::take};

use crate::{
    offset::Offset,
    syntax::{OrRemaining, Rule, RULE_COUNT},
    tracker::{TError, TFunc, TInput, TResult},
};

pub struct LexemeP<const L: usize>;

impl<const L: usize> TFunc for LexemeP<L> {
    fn parse(mut t: TInput) -> TResult {
        if t.lexeme.or_remaining(&mut t.error.remaining).is_none() {
            return Err(t.error);
        }
        let lexeme = t.lexeme.as_mut().unwrap();
        t.error.sensitive_length = max(t.error.sensitive_length, t.length + lexeme.length);
        if lexeme.kind.mask() & L != 0 {
            for rule in &mut lexeme.rules {
                // could possibly also ignore failed rules
                let rule = take(rule);
                if let Some(next) = rule.next {
                    t.error.remaining.push(next)
                }
            }
            t.length += lexeme.length;
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
        if t.lexeme.or_remaining(&mut t.error.remaining).is_none() {
            return Err(t.error);
        }
        let rules = &mut t.lexeme.as_mut().unwrap().rules as *mut [Rule; RULE_COUNT];
        let rules = unsafe { &mut *rules };
        if rules[R].sensitive_length == Offset::default() {
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
            assert!(rules[R].next.is_none());
            rules[R] = Rule {
                sensitive_length: tracker.error.sensitive_length,
                length: tracker.length,
                next: tracker.lexeme.take(),
            };
            t.error.remaining = tracker.error.remaining;
        };

        t.error.sensitive_length = max(
            t.error.sensitive_length,
            t.length + rules[R].sensitive_length,
        );
        if rules[R].length != Offset::default() {
            for rule in rules.iter_mut().take(R) {
                let rule = take(rule);
                if let Some(next) = rule.next {
                    t.error.remaining.push(next)
                }
            }
            t.length += rules[R].length;
            t.lexeme = &mut rules[R].next;
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
