use std::rc::Rc;

#[derive(Debug)]
pub struct Entry {
    pub attr: Term,
    pub val: Term,
}

#[derive(Debug)]
pub enum Term {
    Num(u64),
    Ident(Rc<String>),
    Op {
        left: Box<Term>,
        op_code: OpCode,
        right: Box<Term>,
    },
    MultiOp {
        head: Box<Term>,
        tail: Vec<(MultiOpCode, Term)>,
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
}

impl Term {
    pub fn new_op(left: Term, op_code: OpCode, right: Term) -> Self {
        Self::Op {
            left: Box::new(left),
            right: Box::new(right),
            op_code,
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

    pub fn new_multi_op(head: Term, tail: Vec<(MultiOpCode, Term)>) -> Self {
        Self::MultiOp {
            head: Box::new(head),
            tail,
        }
    }
}

#[derive(Debug)]
pub enum OpCode {
    App,
    Or,
    And,
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
    Func,
}

#[derive(Debug)]
pub enum MultiOpCode {
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}
