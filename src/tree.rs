use std::{cell::Cell, fmt::Debug};

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

impl<'a> Debug for TreeTerm<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeTerm::Num(num) => f.write_str(format!("Num({})", num).as_str()),
            TreeTerm::Var(index) => f.write_str(format!("Var({})", index).as_str()),
            TreeTerm::Ite(i, [t, e]) => f
                .debug_tuple("Ite")
                .field(&i.get())
                .field(&t.get())
                .field(&e.get())
                .finish(),
            TreeTerm::App(func, arg) => f
                .debug_tuple("App")
                .field(&func.get())
                .field(&arg.get())
                .finish(),
            TreeTerm::Op(op, [l, r]) => f
                .debug_tuple("Op")
                .field(op)
                .field(&l.get())
                .field(&r.get())
                .finish(),
            TreeTerm::Link(None) => f.write_str("Error"),
            TreeTerm::Link(_) => f.write_str("..."),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TreeBuilder<'a, 'b> {
    pub args: &'b Arg<'b, Combinator<'b>>,
    pub prev: &'b Env<'b, &'b Vec<Option<&'a Cell<TreeTerm<'a>>>>, &'a Cell<TreeTerm<'a>>>,
    pub arena: &'a Arena<TreeTerm<'a>>,
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

pub struct Convert<'a> {
    val: &'a Cell<TreeTerm<'a>>,
    total: bool,
}

impl<'a> Debug for Convert<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Convert")
            .field("val", &self.val.get())
            .field("total", &self.total)
            .finish()
    }
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
                if let Some((old_args, value)) = state.args.pop() {
                    let no_arg = Arg::new();
                    let mut args = old_args;

                    state.args = &no_arg;
                    let test_value = state.convert(value);

                    let mut app = {
                        let test_func = state.convert(comb);
                        vec![test_func, test_value]
                    };

                    while let Some((new_args, value)) = args.pop() {
                        let test_value = state.convert(value);
                        app.push(test_value);
                        args = new_args;
                    }

                    let mut signature = Vec::new();
                    for arg in &app {
                        if arg.total {
                            signature.push(None)
                        } else {
                            signature.push(Some(arg.val))
                        }
                    }

                    if let Some(mut result) = state.prev.get(&signature) {
                        for arg in &app {
                            if arg.total {
                                result = state.alloc(TreeTerm::App(result, arg.val))
                            }
                        }
                        Convert::total(result)
                    } else {
                        let result = state.alloc(TreeTerm::Link(None));
                        let new_prev = state.prev.insert(&signature, result);
                        state.prev = &new_prev;
                        state.args = old_args;

                        let final_app = if app[1].total {
                            let func = state.convert(Combinator {
                                term: body.as_ref(),
                                local: &comb.local.push(name.as_ref()),
                                scope: comb.scope,
                            });
                            Convert {
                                val: state.alloc(TreeTerm::App(func.val, app[1].val)),
                                total: func.total,
                            }
                        } else {
                            state.convert(Combinator {
                                term: body.as_ref(),
                                local: comb.local,
                                scope: &comb.scope.insert(name.as_ref(), value),
                            })
                        };

                        result.replace(TreeTerm::Link(Some(final_app.val)));
                        final_app
                    }
                } else {
                    let func = state.convert(Combinator {
                        term: body.as_ref(),
                        local: &comb.local.push(name.as_ref()),
                        scope: comb.scope,
                    });
                    Convert::partial(func.val)
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
            CoreTerm::Op { left, op, right } => {
                assert!(state.args.pop().is_none());
                let left = state.convert(comb.make(left));
                let right = state.convert(comb.make(right));
                assert!(left.total && right.total);
                Convert::total(state.alloc(TreeTerm::Op(*op, [left.val, right.val])))
            }
            CoreTerm::Ite {
                cond,
                then,
                otherwise,
            } => {
                let then = state.convert(comb.make(then));
                let otherwise = state.convert(comb.make(otherwise));
                let no_arg = Arg::new();
                state.args = &no_arg;
                let cond = state.convert(comb.make(cond));
                assert!(cond.total);
                assert!(then.total == otherwise.total);
                Convert {
                    val: self.alloc(TreeTerm::Ite(cond.val, [then.val, otherwise.val])),
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
