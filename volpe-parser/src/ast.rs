use std::rc::Rc;

#[derive(Debug)]
pub struct Expr {
    pub body: Vec<Stmt>,
    pub ret: Box<Option<Term>>,
}

impl Expr {
    pub fn new(body: Vec<Stmt>, ret: Option<Term>) -> Self {
        Self {
            body,
            ret: Box::new(ret),
        }
    }
}
#[derive(Debug)]
pub struct Stmt {
    pub var: Vec<Term>,
    pub val: Term,
}

#[derive(Debug)]
pub struct Entry {
    pub attr: Term,
    pub val: Term,
}

#[derive(Debug)]
pub struct Op {
    pub left: Box<Term>,
    pub right: Box<Term>,
    pub op_code: OpCode,
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
    Expr(Expr),
    Matrix(Vec<Vec<Term>>),
    Object(Vec<Entry>),
    Tuple(Vec<Term>),
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
