use std::{cell::Cell, collections::HashMap};

use typed_arena::Arena;
use volpe_parser::ast::{BoolOp, CmpOp, IntOp, Op};

use crate::{
    combinator::{Combinator, CombinatorTerm},
    core::CoreTerm,
};

pub enum UnitTerm<'a> {
    Unit,
    Assert(&'a Cell<BoolTerm<'a>>, &'a Cell<UnitTerm<'a>>),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<UnitTerm<'a>>; 2]),
    Link(Option<&'a Cell<UnitTerm<'a>>>),
}
pub enum BoolTerm<'a> {
    Bool(bool),
    BoolOp(BoolOp, [&'a Cell<BoolTerm<'a>>; 2]),
    CmpOp(CmpOp, [&'a Cell<IntTerm<'a>>; 2]),
    Assert(&'a Cell<BoolTerm<'a>>, &'a Cell<BoolTerm<'a>>),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<BoolTerm<'a>>; 2]),
    Link(Option<&'a Cell<BoolTerm<'a>>>),
}

pub enum IntTerm<'a> {
    Int(i64),
    IntOp(IntOp, [&'a Cell<IntTerm<'a>>; 2]),
    Assert(&'a Cell<BoolTerm<'a>>, &'a Cell<IntTerm<'a>>),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<IntTerm<'a>>; 2]),
    Link(Option<&'a Cell<IntTerm<'a>>>),
}

struct CoreTermBuilder<'a> {
    unit_arena: Arena<UnitTerm<'a>>,
    bool_arena: Arena<BoolTerm<'a>>,
    int_arena: Arena<IntTerm<'a>>,
}

impl<'a> CoreTermBuilder<'a> {
    fn new() -> Self {
        Self {
            unit_arena: Arena::new(),
            bool_arena: Arena::new(),
            int_arena: Arena::new(),
        }
    }

    fn unit<'b>(
        &self,
        term: CombinatorTerm<'b>,
        args: Vec<&CombinatorTerm<'b>>,
    ) -> &'a Cell<UnitTerm> {
        let mut args = args;
        match Combinator::from(term) {
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => Cell::from_mut(self.unit_arena.alloc(
                if Combinator::from(otherwise.clone()) == Combinator::Unreachable {
                    UnitTerm::Assert(self.bool(cond, args.clone()), self.unit(then, args))
                } else {
                    UnitTerm::Ite(
                        self.bool(cond, args.clone()),
                        [self.unit(then, args.clone()), self.unit(otherwise, args)],
                    )
                },
            )),
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let val = args.pop().unwrap();
                body.scope.insert(name.term.as_str().unwrap(), val);
                self.unit(body, args)
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                args.push(&val);
                self.unit(func, args)
            }
            _ => unimplemented!(),
        }
    }

    fn bool<'b>(
        &self,
        term: CombinatorTerm<'b>,
        args: Vec<&CombinatorTerm<'b>>,
    ) -> &'a Cell<BoolTerm> {
        let mut args = args;
        match Combinator::from(term) {
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => Cell::from_mut(self.bool_arena.alloc(
                if Combinator::from(otherwise.clone()) == Combinator::Unreachable {
                    BoolTerm::Assert(self.bool(cond, args.clone()), self.bool(then, args))
                } else {
                    BoolTerm::Ite(
                        self.bool(cond, args.clone()),
                        [self.bool(then, args.clone()), self.bool(otherwise, args)],
                    )
                },
            )),
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let val = args.pop().unwrap();
                body.scope.insert(name.term.as_str().unwrap(), &val);
                self.bool(body, args)
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                args.push(&val);
                self.bool(func, args)
            }
            Combinator::Op {
                left,
                op: Op::Bool(op),
                right,
            } => Cell::from_mut(self.bool_arena.alloc(BoolTerm::BoolOp(
                op,
                [self.bool(left, args.clone()), self.bool(right, args)],
            ))),
            Combinator::Op {
                left,
                op: Op::Cmp(op),
                right,
            } => Cell::from_mut(self.bool_arena.alloc(BoolTerm::CmpOp(
                op,
                [self.int(left, args.clone()), self.int(right, args)],
            ))),
            _ => unimplemented!(),
        }
    }

    fn int<'b>(
        &self,
        term: CombinatorTerm<'b>,
        args: Vec<&CombinatorTerm<'b>>,
    ) -> &'a Cell<IntTerm> {
        let mut args = args;
        match Combinator::from(term) {
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => Cell::from_mut(self.int_arena.alloc(
                if Combinator::from(otherwise.clone()) == Combinator::Unreachable {
                    IntTerm::Assert(self.bool(cond, args.clone()), self.int(then, args))
                } else {
                    IntTerm::Ite(
                        self.bool(cond, args.clone()),
                        [self.int(then, args.clone()), self.int(otherwise, args)],
                    )
                },
            )),
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let val = args.pop().unwrap();
                body.scope.insert(name.term.as_str().unwrap(), &val);
                self.int(body, args)
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                args.push(&val);
                self.int(func, args)
            }
            Combinator::Op {
                left,
                op: Op::Int(op),
                right,
            } => Cell::from_mut(self.int_arena.alloc(IntTerm::IntOp(
                op,
                [self.int(left, args.clone()), self.int(right, args)],
            ))),
            Combinator::Num(num) => Cell::from_mut(self.int_arena.alloc(IntTerm::Int(num as i64))),
            _ => unimplemented!(),
        }
    }
}

mod tests {
    use volpe_parser::parser::ExprParser;

    use super::*;

    #[test]
    fn test_unit() {
        let builder = CoreTermBuilder::new();
        let unit = CombinatorTerm {
            term: &CoreTerm::Unreachable,
            scope: HashMap::new(),
        };
        builder.unit(
            CombinatorTerm {
                term: &ExprParser::new().parse("x.(x x) x.(x x)").unwrap().into(),
                scope: HashMap::new(),
            },
            vec![&unit],
        );
    }
}
