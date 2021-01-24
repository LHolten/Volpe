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

#[derive(Clone)]
struct State<'a, 'b> {
    args: Vec<&'b CombinatorTerm<'b>>,
    unit: HashMap<&'b CombinatorTerm<'b>, &'a Cell<UnitTerm<'a>>>,
    bool: HashMap<&'b CombinatorTerm<'b>, &'a Cell<BoolTerm<'a>>>,
    int: HashMap<&'b CombinatorTerm<'b>, &'a Cell<IntTerm<'a>>>,
}

impl<'a> CoreTermBuilder<'a> {
    fn new() -> Self {
        Self {
            unit_arena: Arena::new(),
            bool_arena: Arena::new(),
            int_arena: Arena::new(),
        }
    }

    fn unit_cell(
        &'a self,
        term: &CombinatorTerm<'_>,
        state: &State<'a, '_>,
    ) -> &'a Cell<UnitTerm<'a>> {
        let mut state = state.clone();
        if let Some(val) = state.unit.get(term) {
            *val
        } else {
            let cell = Cell::from_mut(self.unit_arena.alloc(UnitTerm::Link(None)));
            state.unit.insert(term, cell);
            cell.set(self.unit(term, state));
            cell
        }
    }

    fn unit(&'a self, term: &CombinatorTerm<'_>, state: State<'a, '_>) -> UnitTerm<'a> {
        let mut state = state;
        match Combinator::from(term) {
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => {
                if Combinator::from(&otherwise) == Combinator::Unreachable {
                    UnitTerm::Assert(self.bool_cell(&cond, &state), self.unit_cell(&then, &state))
                } else {
                    UnitTerm::Ite(
                        self.bool_cell(&cond, &state),
                        [
                            self.unit_cell(&then, &state),
                            self.unit_cell(&otherwise, &state),
                        ],
                    )
                }
            }
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let val = state.args.pop().unwrap();
                body.scope.insert(name.term.as_str().unwrap(), val);
                if let Some(val) = state.unit.get(&body) {
                    UnitTerm::Link(Some(*val))
                } else {
                    self.unit(&body, state)
                }
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                state.args.push(&val);
                self.unit(&func, state)
            }
            _ => unimplemented!(),
        }
    }

    fn bool_cell(
        &'a self,
        term: &CombinatorTerm<'_>,
        state: &State<'a, '_>,
    ) -> &'a Cell<BoolTerm<'a>> {
        let mut state = state.clone();
        if let Some(val) = state.bool.get(term) {
            *val
        } else {
            let cell = Cell::from_mut(self.bool_arena.alloc(BoolTerm::Link(None)));
            state.bool.insert(term, cell);
            cell.set(self.bool(term, state));
            cell
        }
    }

    fn bool(&'a self, term: &CombinatorTerm<'_>, state: State<'a, '_>) -> BoolTerm<'a> {
        let mut state = state;
        match Combinator::from(term) {
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => {
                if Combinator::from(&otherwise) == Combinator::Unreachable {
                    BoolTerm::Assert(self.bool_cell(&cond, &state), self.bool_cell(&then, &state))
                } else {
                    BoolTerm::Ite(
                        self.bool_cell(&cond, &state),
                        [
                            self.bool_cell(&then, &state),
                            self.bool_cell(&otherwise, &state),
                        ],
                    )
                }
            }
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let val = state.args.pop().unwrap();
                body.scope.insert(name.term.as_str().unwrap(), val);
                if let Some(val) = state.bool.get(&body) {
                    BoolTerm::Link(Some(*val))
                } else {
                    self.bool(&body, state)
                }
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                state.args.push(&val);
                self.bool(&func, state)
            }
            Combinator::Op {
                left,
                op: Op::Bool(op),
                right,
            } => BoolTerm::BoolOp(
                op,
                [
                    self.bool_cell(&left, &state),
                    self.bool_cell(&right, &state),
                ],
            ),
            Combinator::Op {
                left,
                op: Op::Cmp(op),
                right,
            } => BoolTerm::CmpOp(
                op,
                [self.int_cell(&left, &state), self.int_cell(&right, &state)],
            ),
            _ => unimplemented!(),
        }
    }

    fn int_cell(
        &'a self,
        term: &CombinatorTerm<'_>,
        state: &State<'a, '_>,
    ) -> &'a Cell<IntTerm<'a>> {
        let mut state = state.clone();
        if let Some(val) = state.int.get(term) {
            *val
        } else {
            let cell = Cell::from_mut(self.int_arena.alloc(IntTerm::Link(None)));
            state.int.insert(term, cell);
            cell.set(self.int(term, state));
            cell
        }
    }

    fn int(&'a self, term: &CombinatorTerm<'_>, state: State<'a, '_>) -> IntTerm {
        let mut state = state;
        match Combinator::from(term) {
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => {
                if Combinator::from(&otherwise) == Combinator::Unreachable {
                    IntTerm::Assert(self.bool_cell(&cond, &state), self.int_cell(&then, &state))
                } else {
                    IntTerm::Ite(
                        self.bool_cell(&cond, &state),
                        [
                            self.int_cell(&then, &state),
                            self.int_cell(&otherwise, &state),
                        ],
                    )
                }
            }
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let val = state.args.pop().unwrap();
                body.scope.insert(name.term.as_str().unwrap(), val);
                if let Some(val) = state.int.get(&body) {
                    IntTerm::Link(Some(*val))
                } else {
                    self.int(&body, state)
                }
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                state.args.push(&val);
                self.int(&func, state)
            }
            Combinator::Op {
                left,
                op: Op::Int(op),
                right,
            } => IntTerm::IntOp(
                op,
                [self.int_cell(&left, &state), self.int_cell(&right, &state)],
            ),
            Combinator::Num(num) => IntTerm::Int(num as i64),
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
        builder.unit_cell(
            &CombinatorTerm {
                term: &ExprParser::new().parse("x.(x x) x.(x x)").unwrap().into(),
                scope: HashMap::new(),
            },
            &State {
                args: vec![&unit],
                unit: HashMap::new(),
                bool: HashMap::new(),
                int: HashMap::new(),
            },
        );
    }
}
