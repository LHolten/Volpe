use std::{
    cmp::max,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::{
    internal::{UpgradeInternal, WithInternal},
    offset::Offset,
    syntax::{Rule, RuleKind, Syntax},
    tracker::{TFunc, TInput, TResult, Tracker},
};

pub struct LexemeP<const L: usize>;

impl<const L: usize> TFunc for LexemeP<L> {
    fn parse(mut t: TInput) -> TResult {
        t.tracker.length = max(t.tracker.length, t.offset + t.lexeme.length);
        if t.lexeme.kind.mask() & L != 0 {
            t.tracker.children.push(Syntax::Lexeme(t.lexeme.clone()));
            t.offset += t.lexeme.length;
            t.lexeme = t.lexeme.next.upgrade().unwrap_or_default();
            Ok(t)
        } else {
            Err(t.tracker)
        }
    }
}

pub struct RuleP<F, const R: usize> {
    f: PhantomData<F>,
}

impl<F: TFunc, const R: usize> TFunc for RuleP<F, R> {
    fn parse(mut t: TInput) -> TResult {
        let rule = t.lexeme.rules[R].upgrade().unwrap_or_else(|| {
            let result = F::parse(TInput {
                lexeme: t.lexeme.clone(),
                offset: Offset::default(),
                tracker: Tracker::default(),
            });
            let rule = match result {
                Ok(input) => Rule {
                    children: input.tracker.children,
                    length: input.tracker.length,
                    kind: RuleKind::from(R),
                    offset: input.offset,
                    next: Rc::downgrade(&input.lexeme),
                },
                Err(tracker) => Rule {
                    children: tracker.children,
                    length: tracker.length,
                    kind: RuleKind::from(R),
                    offset: Offset::default(),
                    next: Weak::default(),
                },
            };
            let rule = Rc::new(rule);
            t.lexeme.rules[R].with(|r| *r = Rc::downgrade(&rule));
            rule
        });

        t.tracker.length = max(t.tracker.length, t.offset + rule.length);
        t.tracker.children.push(Syntax::Rule(rule.clone()));
        if let Some(lexeme) = rule.next.upgrade() {
            t.offset += rule.offset;
            t.lexeme = lexeme;
            Ok(t)
        } else {
            Err(t.tracker)
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
        let pos = t.lexeme.clone();
        let offset = t.offset;
        F::parse(t)
            .and_then(|t| {
                if t.offset == offset {
                    Err(t.tracker)
                } else {
                    Self::parse(t)
                }
            })
            .or_else(|tracker| {
                Ok(TInput {
                    lexeme: pos,
                    offset,
                    tracker,
                })
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
        let pos = t.lexeme.clone();
        let offset = t.offset;
        match F::parse(t) {
            result @ Ok(_) => result,
            Err(tracker) => G::parse(TInput {
                lexeme: pos,
                offset,
                tracker,
            }),
        }
    }
}

pub struct Id;

impl TFunc for Id {
    fn parse(t: TInput) -> TResult {
        Ok(t)
    }
}
