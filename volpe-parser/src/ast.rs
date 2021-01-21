#[derive(Debug)]
pub struct Entry {
    pub attr: Term,
    pub val: Term,
}

#[derive(Debug)]
pub enum Term {
    Num(u64),
    Ident(String),
    Op {
        left: Box<Term>,
        op: Op,
        right: Box<Term>,
    },
    MultiOp {
        head: Box<Term>,
        tail: Vec<(Op, Term)>,
    },
    Astmt {
        var: Box<Term>,
        val: Box<Term>,
        next: Box<Term>,
    },
    Stmt {
        var: Vec<Term>,
        val: Box<Term>,
        next: Box<Term>,
    },
    Ite {
        cond: Box<Term>,
        then: Box<Term>,
        otherwise: Box<Term>,
    },
    Matrix(Vec<Vec<Term>>),
    Object(Vec<Entry>),
    Tuple(Vec<Term>),
    Unreachable,
}

impl Term {
    pub fn new_op(left: Term, op_code: Op, right: Term) -> Self {
        Self::Op {
            left: Box::new(left),
            right: Box::new(right),
            op: op_code,
        }
    }

    pub fn new_stmt(var: Vec<Term>, val: Term, next: Term) -> Self {
        Self::Stmt {
            var,
            val: Box::new(val),
            next: Box::new(next),
        }
    }

    pub fn new_astmt(var: Term, val: Term, next: Term) -> Self {
        Self::Astmt {
            var: Box::new(var),
            val: Box::new(val),
            next: Box::new(next),
        }
    }

    pub fn new_ite(cond: Term, then: Term, otherwise: Option<Term>) -> Self {
        Self::Ite {
            cond: Box::new(cond),
            then: Box::new(then),
            otherwise: Box::new(otherwise.unwrap_or(Term::Unreachable)),
        }
    }

    pub fn new_multi_op(head: Term, tail: Vec<(Op, Term)>) -> Self {
        Self::MultiOp {
            head: Box::new(head),
            tail,
        }
    }
}

impl AsRef<Term> for Term {
    fn as_ref(&self) -> &Term {
        &self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Bool(BoolOp),
    Int(IntOp),
    Cmp(CmpOp),
    Func,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitOr,
    BitAnd,
    BitXor,
    BitShl,
    BitShr,
}
