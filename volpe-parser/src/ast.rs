use std::rc::Rc;

pub struct Expr(Vec<Stmt>);
pub struct Stmt {
    var: Vec<Op>,
    val: Op,
}

pub struct Column(Vec<Row>);
pub struct Row(Vec<Op>);

pub struct Obj(Vec<Entry>);
pub struct Entry {
    attr: Op,
    val: Op,
}

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

pub enum Term {
    Num(u64),
    Ident(Rc<String>),
    Op(Op),
    EqualOp(MultiOp<EqualOp>),
    CmpOp(MultiOp<CmpOp>),
    Expr(Expr),
    Column(Column),
    Obj(Obj),
}

pub enum EqualOp {
    Equal,
    Unequal,
}

pub enum CmpOp {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}
