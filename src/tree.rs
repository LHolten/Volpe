use std::cell::Cell;

use typed_arena::Arena;
use volpe_parser::ast::Op;

use crate::{
    core::CoreTerm,
    state::{Arg, Env},
};

#[derive(Clone, Copy, PartialEq)]
pub enum TreeTerm<'a> {
    Num(u64),
    Var(usize),
    Ite(&'a Cell<TreeTerm<'a>>, [&'a Cell<TreeTerm<'a>>; 2]),
    App(&'a Cell<TreeTerm<'a>>, &'a Cell<TreeTerm<'a>>),
    Op(Op, [&'a Cell<TreeTerm<'a>>; 2]),
    Link(Option<&'a Cell<TreeTerm<'a>>>),
}

#[derive(Clone, Copy)]
struct TreeBuilder<'a, 'b> {
    args: &'b Arg<'b, Combinator<'b>>,
    prev: &'b Env<'b, &'a Cell<TreeTerm<'a>>, &'a Cell<TreeTerm<'a>>>,
    arena: &'a Arena<TreeTerm<'a>>,
}

#[derive(Clone, Copy)]
pub struct Combinator<'b> {
    pub term: &'b CoreTerm,
    pub local: &'b Arg<'b, &'b CoreTerm>,
    pub scope: &'b Env<'b, &'b CoreTerm, Combinator<'b>>,
}

impl<'b> Combinator<'b> {
    pub fn make<T: AsRef<CoreTerm>>(mut self, term: &'b T) -> Self {
        self.term = term.as_ref();
        self
    }
}

struct Convert<'a> {
    val: &'a Cell<TreeTerm<'a>>,
    total: bool,
}

impl<'a> Convert<'a> {
    fn total(val: &'a Cell<TreeTerm<'a>>) -> Self {
        Self { val, total: true }
    }
    fn partial(val: &'a Cell<TreeTerm<'a>>) -> Self {
        Self { val, total: false }
    }
}

impl<'a, 'b> TreeBuilder<'a, 'b> {
    fn alloc(self, term: TreeTerm<'a>) -> &'a Cell<TreeTerm<'a>> {
        Cell::from_mut(self.arena.alloc(term))
    }

    pub fn convert(self, comb: Combinator<'b>) -> Convert<'a> {
        let mut state = self;
        match comb.term {
            CoreTerm::Ident(_) => {
                if let Some(index) = comb.local.find(comb.term) {
                    Convert::total(state.alloc(TreeTerm::Var(index)))
                } else {
                    state.convert(comb.scope.get(&comb.term).unwrap())
                }
            }
            CoreTerm::Num(num) => Convert::total(self.alloc(TreeTerm::Num(*num))),
            CoreTerm::Unreachable => Convert::total(self.alloc(TreeTerm::Link(None))),
            CoreTerm::Op {
                left: name,
                op: Op::Func,
                right: body,
            } => {
                let test_func = state.convert(Combinator {
                    term: body.as_ref(),
                    local: &comb.local.push(name.as_ref()),
                    scope: comb.scope,
                });
                if let Some((args, value)) = state.args.pop() {
                    if let Some(val) = state.prev.get(test_func.val) {
                        Convert::total(val)
                    } else {
                        let result = state.alloc(TreeTerm::Link(None));
                        let state = TreeBuilder {
                            args,
                            prev: &state.prev.insert(test_func.val, result),
                            arena: state.arena,
                        };
                        let test_value = state.convert(value);
                        if test_value.total {
                            result.set(TreeTerm::App(test_func.val, test_value.val));
                            Convert {
                                val: result,
                                total: test_func.total,
                            }
                        } else {
                            let func = state.convert(Combinator {
                                term: body.as_ref(),
                                local: comb.local,
                                scope: &comb.scope.insert(name.as_ref(), value),
                            });
                            result.set(TreeTerm::Link(Some(func.val)));
                            Convert {
                                val: result,
                                total: func.total,
                            }
                        }
                    }
                } else {
                    Convert::partial(test_func.val)
                }
            }
            CoreTerm::Op {
                left: func,
                op: Op::App,
                right: value,
            } => {
                let args = state.args.push(comb.make(value));
                state.args = &args;
                state.convert(comb.make(func))
            }
            CoreTerm::Op { left, op, right } => Convert::total(state.alloc(TreeTerm::Op(
                *op,
                [
                    state.convert(comb.make(left)).val,
                    state.convert(comb.make(right)).val,
                ],
            ))),
            CoreTerm::Ite {
                cond,
                then,
                otherwise,
            } => {
                let then = state.convert(comb.make(then));
                Convert {
                    val: self.alloc(TreeTerm::Ite(
                        state.convert(comb.make(cond)).val,
                        [then.val, state.convert(comb.make(otherwise)).val],
                    )),
                    total: then.total,
                }
            }
            CoreTerm::Matrix(_) => unimplemented!(),
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
            prev: &Env::new(),
            arena: &Arena::new(),
        };
        builder.convert(Combinator {
            term: &ExprParser::new().parse("x.(x x) x.(x x)").unwrap().into(),
            scope: &Env::new(),
            local: &Arg::new(),
        });
    }

    #[test]
    fn test_int() {
        let builder = TreeBuilder {
            args: &Arg::new(),
            prev: &Env::new(),
            arena: &Arena::new(),
        };
        builder.convert(Combinator {
            term: &ExprParser::new()
                .parse(
                    "
                    fix := f.(x.(f (x x)) x.(f (x x)));
                    fix rec.(rec + 1)
                    ",
                )
                .unwrap()
                .into(),
            scope: &Env::new(),
            local: &Arg::new(),
        });
    }

    #[test]
    fn test_complex() {
        let builder = TreeBuilder {
            args: &Arg::new(),
            prev: &Env::new(),
            arena: &Arena::new(),
        };
        builder.convert(Combinator {
            term: &ExprParser::new()
                .parse(
                    "
                        fix := f.(x.(f (x x)) x.(f (x x)));
                        fix rec.x.(x == 0 => 0; rec x - 1) 10
                        ",
                )
                .unwrap()
                .into(),
            scope: &Env::new(),
            local: &Arg::new(),
        });
    }
}
