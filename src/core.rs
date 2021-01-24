use volpe_parser::ast::{BoolOp, CmpOp, IntOp, Op, Term};

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl CoreTerm {
    pub fn new_op(left: CoreTerm, op: Op, right: CoreTerm) -> Self {
        Self::Op {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Self::Ident(name) = self {
            Some(name.as_str())
        } else {
            None
        }
    }
}

impl AsRef<CoreTerm> for CoreTerm {
    fn as_ref(&self) -> &CoreTerm {
        &self
    }
}

impl<T: AsRef<Term>> From<T> for CoreTerm {
    fn from(term: T) -> Self {
        match term.as_ref() {
            Term::Num(val) => CoreTerm::Num(*val),
            Term::Ident(name) => CoreTerm::Ident(name.clone()),
            Term::Op { left, op, right } => CoreTerm::new_op(left.into(), *op, right.into()),
            Term::MultiOp { head, tail } => {
                let ((op, next), rest) = tail.split_first().unwrap();
                let next = CoreTerm::from(next);
                let mut result = CoreTerm::new_op(head.into(), *op, next.clone());
                let mut prev = next;
                for (op, next) in rest {
                    let next = CoreTerm::from(next);
                    result = CoreTerm::new_op(
                        result,
                        Op::Bool(BoolOp::And),
                        CoreTerm::new_op(prev, *op, next.clone()),
                    );
                    prev = next;
                }
                result
            }
            Term::Stmt { var, val, next } => {
                let mut func = next.into();
                for arg in var.iter().rev() {
                    func = CoreTerm::new_op(arg.into(), Op::Func, func)
                }
                CoreTerm::new_op(val.into(), Op::App, func)
            }
            Term::Astmt { var, val, next } => CoreTerm::new_op(
                CoreTerm::new_op(var.into(), Op::Func, next.into()),
                Op::App,
                val.into(),
            ),
            Term::Ite {
                cond,
                then,
                otherwise,
            } => CoreTerm::Ite {
                cond: Box::new(cond.into()),
                then: Box::new(then.into()),
                otherwise: Box::new(otherwise.into()),
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
                        cond: Box::new(CoreTerm::new_op(
                            CoreTerm::Ident("$".to_string()),
                            Op::Cmp(CmpOp::Equal),
                            (&next.attr).into(),
                        )),
                        then: Box::new((&next.val).into()),
                        otherwise: Box::new(prev),
                    }
                }
                CoreTerm::new_op(CoreTerm::Ident("$".to_string()), Op::Func, prev)
            }
            Term::Tuple(values) => {
                let mut prev = CoreTerm::Ident("$".to_string());
                for val in values {
                    prev = CoreTerm::new_op(prev, Op::App, val.into())
                }
                CoreTerm::new_op(CoreTerm::Ident("$".to_string()), Op::Func, prev)
            }
            Term::Unreachable => CoreTerm::Unreachable,
        }
    }
}
