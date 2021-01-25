use std::cell::Cell;

use typed_arena::Arena;
use volpe_parser::ast::{BoolOp, CmpOp, IntOp, Op};

use crate::{
    combinator::{Combinator, CombinatorTerm},
    state::{Arg, Env},
};

pub enum UnitTerm<'a> {
    // Unit,
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<UnitTerm<'a>>; 2]),
    Link(Option<&'a Cell<UnitTerm<'a>>>),
}
pub enum BoolTerm<'a> {
    Bool(bool),
    BoolOp(BoolOp, [&'a Cell<BoolTerm<'a>>; 2]),
    CmpOp(CmpOp, [&'a Cell<IntTerm<'a>>; 2]),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<BoolTerm<'a>>; 2]),
    Link(Option<&'a Cell<BoolTerm<'a>>>),
}

pub enum IntTerm<'a> {
    Int(i64),
    IntOp(IntOp, [&'a Cell<IntTerm<'a>>; 2]),
    Ite(&'a Cell<BoolTerm<'a>>, [&'a Cell<IntTerm<'a>>; 2]),
    Link(Option<&'a Cell<IntTerm<'a>>>),
}

struct ArenaStore<'a> {
    unit: Arena<UnitTerm<'a>>,
    bool: Arena<BoolTerm<'a>>,
    int: Arena<IntTerm<'a>>,
}

impl<'a> ArenaStore<'a> {
    fn new() -> Self {
        Self {
            unit: Arena::new(),
            bool: Arena::new(),
            int: Arena::new(),
        }
    }
}

#[derive(Clone, Copy)]
struct TreeBuilder<'a, 'b> {
    args: &'b Arg<'b, &'b CombinatorTerm<'b>>,
    unit: &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<UnitTerm<'a>>>,
    bool: &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<BoolTerm<'a>>>,
    int: &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<IntTerm<'a>>>,
    arena: &'a ArenaStore<'a>,
}

trait TreeTerm<'a>: Sized {
    fn env<'b, 'c>(
        builder: &'c mut TreeBuilder<'a, 'b>,
    ) -> &'c mut &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<Self>>;
    fn arena<'b>(builder: TreeBuilder<'a, 'b>) -> &'a Arena<Self>;
    fn ite<'b>(cond: &'a Cell<BoolTerm<'a>>, then_else: [&'a Cell<Self>; 2]) -> Self;
    fn link<'b>(value: Option<&'a Cell<Self>>) -> Self;
    fn convert<'b>(state: TreeBuilder<'a, 'b>, term: Combinator<'b>) -> Self;
}

impl<'a> TreeTerm<'a> for UnitTerm<'a> {
    fn env<'b, 'c>(
        builder: &'c mut TreeBuilder<'a, 'b>,
    ) -> &'c mut &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<Self>> {
        &mut builder.unit
    }

    fn arena<'b>(builder: TreeBuilder<'a, 'b>) -> &'a Arena<Self> {
        &builder.arena.unit
    }

    fn ite<'b>(cond: &'a Cell<BoolTerm<'a>>, then_else: [&'a Cell<Self>; 2]) -> Self {
        Self::Ite(cond, then_else)
    }

    fn link<'b>(value: Option<&'a Cell<Self>>) -> Self {
        Self::Link(value)
    }

    fn convert<'b>(_state: TreeBuilder<'a, 'b>, _term: Combinator<'b>) -> Self {
        unimplemented!()
    }
}

impl<'a> TreeTerm<'a> for BoolTerm<'a> {
    fn env<'b, 'c>(
        builder: &'c mut TreeBuilder<'a, 'b>,
    ) -> &'c mut &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<Self>> {
        &mut builder.bool
    }

    fn arena<'b>(builder: TreeBuilder<'a, 'b>) -> &'a Arena<Self> {
        &builder.arena.bool
    }

    fn ite<'b>(cond: &'a Cell<BoolTerm<'a>>, then_else: [&'a Cell<Self>; 2]) -> Self {
        Self::Ite(cond, then_else)
    }

    fn link<'b>(value: Option<&'a Cell<Self>>) -> Self {
        Self::Link(value)
    }

    fn convert<'b>(state: TreeBuilder<'a, 'b>, term: Combinator<'b>) -> Self {
        match term {
            Combinator::Op {
                left,
                op: Op::Bool(op),
                right,
            } => BoolTerm::BoolOp(op, [state.convert_cell(&left), state.convert_cell(&right)]),
            Combinator::Op {
                left,
                op: Op::Cmp(op),
                right,
            } => BoolTerm::CmpOp(op, [state.convert_cell(&left), state.convert_cell(&right)]),
            Combinator::Num(num) => BoolTerm::Bool(num != 0),
            _ => unimplemented!(),
        }
    }
}

impl<'a> TreeTerm<'a> for IntTerm<'a> {
    fn env<'b, 'c>(
        builder: &'c mut TreeBuilder<'a, 'b>,
    ) -> &'c mut &'b Env<'b, &'b CombinatorTerm<'b>, &'a Cell<Self>> {
        &mut builder.int
    }

    fn arena<'b>(builder: TreeBuilder<'a, 'b>) -> &'a Arena<Self> {
        &builder.arena.int
    }

    fn ite<'b>(cond: &'a Cell<BoolTerm<'a>>, then_else: [&'a Cell<Self>; 2]) -> Self {
        Self::Ite(cond, then_else)
    }

    fn link<'b>(value: Option<&'a Cell<Self>>) -> Self {
        Self::Link(value)
    }

    fn convert<'b>(state: TreeBuilder<'a, 'b>, term: Combinator<'b>) -> Self {
        match term {
            Combinator::Op {
                left,
                op: Op::Int(op),
                right,
            } => IntTerm::IntOp(op, [state.convert_cell(&left), state.convert_cell(&right)]),
            Combinator::Num(num) => IntTerm::Int(num as i64),
            _ => unimplemented!(),
        }
    }
}

impl<'a, 'b> TreeBuilder<'a, 'b> {
    fn convert_cell<T: TreeTerm<'a>>(self, term: &'b CombinatorTerm<'b>) -> &'a Cell<T> {
        let mut state = self;
        if let Some(val) = T::env(&mut state).get(term) {
            val
        } else {
            let cell = Cell::from_mut(T::arena(state).alloc(T::link(None)));
            let env = T::env(&mut state).insert(term, cell);
            *T::env(&mut state) = &env;
            cell.set(state.convert(term));
            cell
        }
    }

    fn convert<T: 'a + TreeTerm<'a>>(self, term: &'b CombinatorTerm<'b>) -> T {
        let mut state = self;
        match Combinator::from(term) {
            Combinator::Unreachable => T::link(None),
            Combinator::Ite {
                cond,
                then,
                otherwise,
            } => T::ite(
                state.convert_cell(&cond),
                [state.convert_cell(&then), state.convert_cell(&otherwise)],
            ),
            Combinator::Ident(_) => unimplemented!(),
            Combinator::Op {
                left: name,
                op: Op::Func,
                right: mut body,
            } => {
                let (args, val) = state.args.pop().unwrap();
                state.args = args;
                let scope = body.scope;
                body.scope = scope.insert(name.term.as_str().unwrap(), Some(val));
                if let Some(val) = T::env(&mut state).get(&body) {
                    T::link(Some(val))
                } else {
                    state.convert(&body)
                }
            }
            Combinator::Op {
                left: func,
                op: Op::App,
                right: val,
            } => {
                let args = state.args.push(&val);
                state.args = &args;
                state.convert(&func)
            }
            other => T::convert(state, other),
        }
    }
}

mod tests {
    use volpe_parser::parser::ExprParser;

    use super::*;

    #[test]
    fn test_unit() {
        let builder = TreeBuilder {
            args: &Arg::new(),
            unit: &Env::new(),
            bool: &Env::new(),
            int: &Env::new(),
            arena: &ArenaStore::new(),
        };
        builder.convert_cell::<UnitTerm>(&CombinatorTerm {
            term: &ExprParser::new().parse("x.(x x) x.(x x)").unwrap().into(),
            scope: Env::new(),
        });
    }

    #[test]
    fn test_int() {
        let builder = TreeBuilder {
            args: &Arg::new(),
            unit: &Env::new(),
            bool: &Env::new(),
            int: &Env::new(),
            arena: &ArenaStore::new(),
        };
        builder.convert_cell::<IntTerm>(&CombinatorTerm {
            term: &ExprParser::new()
                .parse(
                    "
                    fix := f.(x.(f (x x)) x.(f (x x)));
                    fix rec.(rec + 1)
                    ",
                )
                .unwrap()
                .into(),
            scope: Env::new(),
        });
    }
}
