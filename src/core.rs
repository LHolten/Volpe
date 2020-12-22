use volpe_parser::ast::{IntOp, Op, Term};

pub enum CoreTerm {
    Num(u64),
    Ident(String),
    Op {
        left: Box<CoreTerm>,
        op: Op,
        right: Box<CoreTerm>,
    },
    Ite {
        cond: Box<CoreTerm>,
        then: Box<CoreTerm>,
        otherwise: Box<CoreTerm>,
    },
    Matrix(Vec<Vec<CoreTerm>>),
    Unreachable,
}

impl From<&Term> for CoreTerm {
    fn from(term: &Term) -> Self {
        match term {
            Term::Num(val) => CoreTerm::Num(*val),
            Term::Ident(name) => CoreTerm::Ident(name.clone()),
            Term::Op { left, op, right } => CoreTerm::Op {
                left: Box::new(left.as_ref().into()),
                op: op.clone(),
                right: Box::new(right.as_ref().into()),
            },
            Term::MultiOp { head, tail } => {
                let mut prev = head.as_ref().into();
                for (op, next) in tail {
                    prev = CoreTerm::Op {
                        left: Box::new(prev),
                        op: op.clone(),
                        right: Box::new(next.into()),
                    }
                }
                prev
            }
            Term::Stmt { var, val, next } => {
                let mut func = next.as_ref().into();
                for arg in var {
                    func = CoreTerm::Op {
                        left: Box::new(arg.into()),
                        op: Op::Func,
                        right: Box::new(func),
                    }
                }
                CoreTerm::Op {
                    left: Box::new(val.as_ref().into()),
                    op: Op::App,
                    right: Box::new(func),
                }
            }
            Term::Ite {
                cond,
                then,
                otherwise,
            } => CoreTerm::Ite {
                cond: Box::new(cond.as_ref().into()),
                then: Box::new(then.as_ref().into()),
                otherwise: Box::new(otherwise.as_ref().into()),
            },
            Term::Matrix(vec) => CoreTerm::Matrix(
                vec.iter()
                    .map(|v| v.iter().map(|v| v.into()).collect())
                    .collect(),
            ),
            Term::Object(entries) => {
                // this code assumes all keys are distinct
                let mut prev = CoreTerm::Unreachable;
                for next in entries {
                    prev = CoreTerm::Ite {
                        cond: Box::new(CoreTerm::Op {
                            left: Box::new(CoreTerm::Ident("$".to_string())),
                            op: Op::Int(IntOp::Equal),
                            right: Box::new((&next.attr).into()),
                        }),
                        then: Box::new((&next.val).into()),
                        otherwise: Box::new(prev),
                    }
                }
                CoreTerm::Op {
                    left: Box::new(CoreTerm::Ident("$".to_string())),
                    op: Op::Func,
                    right: Box::new(prev),
                }
            }
            Term::Tuple(values) => {
                let mut prev = CoreTerm::Ident("$".to_string());
                for val in values {
                    prev = CoreTerm::Op {
                        left: Box::new(prev),
                        op: Op::App,
                        right: Box::new(val.into()),
                    }
                }
                CoreTerm::Op {
                    left: Box::new(CoreTerm::Ident("$".to_string())),
                    op: Op::Func,
                    right: Box::new(prev),
                }
            }
            Term::Unreachable => CoreTerm::Unreachable,
        }
    }
}
