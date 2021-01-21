use std::{cell::Cell, collections::HashMap};

use typed_arena::Arena;
use volpe_parser::ast::{BoolOp, CmpOp, IntOp, Op};

use crate::core::CoreTerm;

pub enum UnitTerm<'a> {
    Unit,
    Assert(&'a Cell<BoolTerm<'a>>, &'a Cell<UnitTerm<'a>>),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<UnitTerm<'a>>; 2]),
}
pub enum BoolTerm<'a> {
    Bool(bool),
    BoolOp(BoolOp, [&'a Cell<BoolTerm<'a>>; 2]),
    CmpOp(CmpOp, [&'a Cell<IntTerm<'a>>; 2]),
    Assert(&'a Cell<BoolTerm<'a>>, &'a Cell<BoolTerm<'a>>),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<BoolTerm<'a>>; 2]),
}

pub enum IntTerm<'a> {
    Int(i64),
    IntOp(IntOp, [&'a Cell<IntTerm<'a>>; 2]),
    Assert(&'a Cell<BoolTerm<'a>>, &'a Cell<IntTerm<'a>>),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<IntTerm<'a>>; 2]),
}

#[derive(Clone)]
struct ScopeItem<'a, 'b> {
    term: &'b CoreTerm,
    scope: Vec<(&'b CoreTerm, ScopeItem<'a, 'b>)>,
    unit: Vec<(Vec<ScopeItem<'a, 'b>>, &'a Cell<UnitTerm<'a>>)>,
    bool: Vec<(Vec<ScopeItem<'a, 'b>>, &'a Cell<BoolTerm<'a>>)>,
    int: Vec<(Vec<ScopeItem<'a, 'b>>, &'a Cell<IntTerm<'a>>)>,
}

impl<'a, 'b> ScopeItem<'a, 'b> {
    fn new(term: &'b impl AsRef<CoreTerm>, scope: Vec<(&'b CoreTerm, ScopeItem<'a, 'b>)>) -> Self {
        Self {
            term: term.as_ref(),
            scope,
            unit: Vec::new(),
            bool: Vec::new(),
            int: Vec::new(),
        }
    }
}

impl<'a, 'b> PartialEq for ScopeItem<'a, 'b> {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term && self.scope == other.scope
    }
}

impl<'a, 'b> Eq for ScopeItem<'a, 'b> {}

struct CoreTermBuilder<'a> {
    unit_arena: Arena<UnitTerm<'a>>,
    bool_arena: Arena<BoolTerm<'a>>,
    int_arena: Arena<IntTerm<'a>>,
}

impl<'a, 'b> CoreTermBuilder<'a> {
    fn new() -> Self {
        Self {
            unit_arena: Arena::new(),
            bool_arena: Arena::new(),
            int_arena: Arena::new(),
        }
    }

    fn unit(
        &self,
        term: &'b impl AsRef<CoreTerm>,
        scope: &mut Vec<(&'b CoreTerm, ScopeItem<'a, 'b>)>,
        args: &mut Vec<ScopeItem<'a, 'b>>,
    ) -> &'a Cell<UnitTerm> {
        match term.as_ref() {
            CoreTerm::Ite {
                cond,
                then,
                otherwise,
            } => Cell::from_mut(self.unit_arena.alloc(
                if otherwise.as_ref() == &CoreTerm::Unreachable {
                    UnitTerm::Assert(self.bool(cond, scope, args), self.unit(then, scope, args))
                } else {
                    UnitTerm::Ite(
                        self.bool(cond, scope, args),
                        [
                            self.unit(then, scope, args),
                            self.unit(otherwise, scope, args),
                        ],
                    )
                },
            )),
            name @ &CoreTerm::Ident(_) => {
                let val = &mut scope.iter_mut().rfind(|(n, _)| n == &name).unwrap().1;
                if let Some(res) = &val.unit.iter().find(|v| &v.0 == args) {
                    res.1
                } else {
                    let unit_term = self.unit(val.term, &mut val.scope, args);
                    val.unit.push((args.clone(), unit_term));
                    unit_term
                }
            }
            CoreTerm::Op {
                left: name,
                op: Op::Func,
                right: body,
            } => {
                scope.push((name.as_ref(), args.pop().unwrap()));
                let unit_term = self.unit(body, scope, args);
                args.push(scope.pop().unwrap().1);
                unit_term
            }
            CoreTerm::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                args.push(ScopeItem::new(val, scope.clone()));
                let unit_term = self.unit(func, scope, args);
                args.pop().unwrap();
                unit_term
            }
            _ => unimplemented!(),
        }
    }

    fn bool(
        &self,
        term: &'b impl AsRef<CoreTerm>,
        scope: &mut Vec<(&'b CoreTerm, ScopeItem<'a, 'b>)>,
        args: &mut Vec<ScopeItem<'a, 'b>>,
    ) -> &'a Cell<BoolTerm> {
        match term.as_ref() {
            CoreTerm::Ite {
                cond,
                then,
                otherwise,
            } => Cell::from_mut(self.bool_arena.alloc(
                if otherwise.as_ref() == &CoreTerm::Unreachable {
                    BoolTerm::Assert(self.bool(cond, scope, args), self.bool(then, scope, args))
                } else {
                    BoolTerm::Ite(
                        self.bool(cond, scope, args),
                        [
                            self.bool(then, scope, args),
                            self.bool(otherwise, scope, args),
                        ],
                    )
                },
            )),
            name @ &CoreTerm::Ident(_) => {
                let val = &mut scope.iter_mut().rfind(|(n, _)| n == &name).unwrap().1;
                if let Some(res) = &val.bool.iter().find(|v| &v.0 == args) {
                    res.1
                } else {
                    let bool_term = self.bool(val.term, &mut val.scope, args);
                    val.bool.push((args.clone(), bool_term));
                    bool_term
                }
            }
            CoreTerm::Op {
                left: name,
                op: Op::Func,
                right: body,
            } => {
                scope.push((name.as_ref(), args.pop().unwrap()));
                let bool_term = self.bool(body, scope, args);
                args.push(scope.pop().unwrap().1);
                bool_term
            }
            CoreTerm::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                args.push(ScopeItem::new(val, scope.clone()));
                let bool_term = self.bool(func, scope, args);
                args.pop().unwrap();
                bool_term
            }
            CoreTerm::Op {
                left,
                op: Op::Bool(op),
                right,
            } => Cell::from_mut(self.bool_arena.alloc(BoolTerm::BoolOp(
                *op,
                [self.bool(left, scope, args), self.bool(right, scope, args)],
            ))),
            CoreTerm::Op {
                left,
                op: Op::Cmp(op),
                right,
            } => Cell::from_mut(self.bool_arena.alloc(BoolTerm::CmpOp(
                *op,
                [self.int(left, scope, args), self.int(right, scope, args)],
            ))),
            _ => unimplemented!(),
        }
    }

    fn int(
        &self,
        term: &'b impl AsRef<CoreTerm>,
        scope: &mut Vec<(&'b CoreTerm, ScopeItem<'a, 'b>)>,
        args: &mut Vec<ScopeItem<'a, 'b>>,
    ) -> &'a Cell<IntTerm> {
        match term.as_ref() {
            CoreTerm::Ite {
                cond,
                then,
                otherwise,
            } => Cell::from_mut(self.int_arena.alloc(
                if otherwise.as_ref() == &CoreTerm::Unreachable {
                    IntTerm::Assert(self.bool(cond, scope, args), self.int(then, scope, args))
                } else {
                    IntTerm::Ite(
                        self.bool(cond, scope, args),
                        [
                            self.int(then, scope, args),
                            self.int(otherwise, scope, args),
                        ],
                    )
                },
            )),
            name @ &CoreTerm::Ident(_) => {
                let val = &mut scope.iter_mut().rfind(|(n, _)| n == &name).unwrap().1;
                if let Some(res) = &val.int.iter().find(|v| &v.0 == args) {
                    res.1
                } else {
                    let int_term = self.int(val.term, &mut val.scope, args);
                    val.int.push((args.clone(), int_term));
                    int_term
                }
            }
            CoreTerm::Op {
                left: name,
                op: Op::Func,
                right: body,
            } => {
                scope.push((name.as_ref(), args.pop().unwrap()));
                let int_term = self.int(body, scope, args);
                args.push(scope.pop().unwrap().1);
                int_term
            }
            CoreTerm::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                args.push(ScopeItem::new(val, scope.clone()));
                let int_term = self.int(func, scope, args);
                args.pop().unwrap();
                int_term
            }
            CoreTerm::Op {
                left,
                op: Op::Int(op),
                right,
            } => Cell::from_mut(self.int_arena.alloc(IntTerm::IntOp(
                *op,
                [self.int(left, scope, args), self.int(right, scope, args)],
            ))),
            CoreTerm::Num(num) => Cell::from_mut(self.int_arena.alloc(IntTerm::Int(*num as i64))),
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
        let mut scope = Vec::new();
        let mut args = vec![ScopeItem {
            term: &CoreTerm::Unreachable,
            scope: Vec::new(),
            unit: vec![(
                Vec::new(),
                Cell::from_mut(builder.unit_arena.alloc(UnitTerm::Unit)),
            )],
            bool: Vec::new(),
            int: Vec::new(),
        }];
        builder.unit(
            &CoreTerm::from(ExprParser::new().parse("1 < 0 => {}").unwrap()),
            &mut scope,
            &mut args,
        );
    }
}
