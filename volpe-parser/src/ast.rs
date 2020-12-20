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
    Assert {
        cond: Box<Term>,
        val: Box<Term>,
    },
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

    pub fn new_ite(cond: Term, then: Term, otherwise: Term) -> Self {
        Self::Ite {
            cond: Box::new(cond),
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }

    pub fn new_multi_op(head: Term, tail: Vec<(Op, Term)>) -> Self {
        Self::MultiOp {
            head: Box::new(head),
            tail,
        }
    }

    pub fn new_assert(cond: Term, val: Term) -> Self {
        Self::Assert {
            cond: Box::new(cond),
            val: Box::new(val),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Op {
    Bool(BoolOp),
    Int(IntOp),
    Func,
    App,
}

#[derive(Debug, Clone)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum IntOp {
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
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
