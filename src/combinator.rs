use std::{fmt::Debug, hash::Hash};

use volpe_parser::ast::Op;

use crate::{core::CoreTerm, state::Env};

#[derive(Clone)]
pub struct CombinatorTerm<'b> {
    pub term: &'b CoreTerm,
    pub scope: Env<'b, &'b str, Option<&'b CombinatorTerm<'b>>>,
}

impl<'b> PartialEq for CombinatorTerm<'b> {
    fn eq(&self, other: &Self) -> bool {
        Combinator::from(self).eq(&Combinator::from(other))
    }
}

impl<'b> Hash for CombinatorTerm<'b> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Combinator::from(self).hash(state)
    }
}

impl<'b> Debug for CombinatorTerm<'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Combinator::from(self).fmt(f)
    }
}

#[derive(Debug, PartialEq, Hash)]
pub enum Combinator<'b> {
    Num(u64),
    Ident(&'b str),
    Op {
        left: CombinatorTerm<'b>,
        op: Op,
        right: CombinatorTerm<'b>,
    },
    Ite {
        cond: CombinatorTerm<'b>,
        then: CombinatorTerm<'b>,
        otherwise: CombinatorTerm<'b>,
    },
    Unreachable,
}

impl<'b> From<&'b CombinatorTerm<'b>> for Combinator<'b> {
    fn from(other: &'b CombinatorTerm<'b>) -> Self {
        match other.term {
            CoreTerm::Num(num) => Combinator::Num(*num),
            CoreTerm::Ident(name) => {
                if let Some(value) = other.scope.get(name.as_str()).unwrap() {
                    Combinator::from(value)
                } else {
                    Combinator::Ident(name.as_str())
                }
            }
            CoreTerm::Op {
                left: name,
                op: Op::Func,
                right: body,
            } => {
                let scope = other.scope.insert(name.as_str().unwrap(), None);
                Combinator::Op {
                    left: CombinatorTerm {
                        term: name.as_ref(),
                        scope: scope.clone(),
                    },
                    op: Op::Func,
                    right: CombinatorTerm {
                        term: body.as_ref(),
                        scope,
                    },
                }
            }
            CoreTerm::Op { left, op, right } => Combinator::Op {
                left: CombinatorTerm {
                    term: left.as_ref(),
                    scope: other.scope.clone(),
                },
                op: *op,
                right: CombinatorTerm {
                    term: right.as_ref(),
                    scope: other.scope.clone(),
                },
            },
            CoreTerm::Ite {
                cond,
                then,
                otherwise,
            } => Combinator::Ite {
                cond: CombinatorTerm {
                    term: cond.as_ref(),
                    scope: other.scope.clone(),
                },
                then: CombinatorTerm {
                    term: then.as_ref(),
                    scope: other.scope.clone(),
                },
                otherwise: CombinatorTerm {
                    term: otherwise.as_ref(),
                    scope: other.scope.clone(),
                },
            },
            CoreTerm::Unreachable => Combinator::Unreachable,
            _ => unimplemented!(),
        }
    }
}
