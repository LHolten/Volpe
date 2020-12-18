use std::rc::Rc;

#[derive(Debug)]
pub struct Stmt {
    pub var: Vec<Term>,
    pub val: Box<Term>,
    pub next: Box<Term>,
}

impl Stmt {
    pub fn new(var: Vec<Term>, val: Term, next: Term) -> Self {
        Self {
            var,
            val: Box::new(val),
            next: Box::new(next),
        }
    }
}

#[derive(Debug)]
pub struct Ite {
    pub cond: Box<Term>,
    pub then: Box<Term>,
    pub otherwise: Box<Term>,
}

impl Ite {
    pub fn new(cond: Term, then: Term, otherwise: Term) -> Self {
        Self {
            cond: Box::new(cond),
            then: Box::new(then),
            otherwise: Box::new(otherwise),
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub attr: Term,
    pub val: Term,
}

#[derive(Debug)]
pub struct Op {
    pub left: Box<Term>,
    pub op_code: OpCode,
    pub right: Box<Term>,
}

impl Op {
    pub fn new(left: Term, op_code: OpCode, right: Term) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            op_code,
        }
    }
}

#[derive(Debug)]
pub struct MultiOp<O> {
    pub head: Box<Term>,
    pub tail: Vec<(O, Term)>,
}

impl<O> MultiOp<O> {
    pub fn new(head: Term, tail: Vec<(O, Term)>) -> Self {
        Self {
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
pub enum Term {
    Num(u64),
    Ident(Rc<String>),
    Op(Op),
    EqualOp(MultiOp<EqualOp>),
    CmpOp(MultiOp<CmpOp>),
    Stmt(Stmt),
    Ite(Ite),
    Matrix(Vec<Vec<Term>>),
    Object(Vec<Entry>),
    Tuple(Vec<Term>),
    Id,
}

#[derive(Debug)]
pub enum EqualOp {
    Equal,
    Unequal,
}

#[derive(Debug)]
pub enum CmpOp {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}
