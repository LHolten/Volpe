use std::rc::Rc;

struct Expr(Vec<Stmt>);
struct Stmt {
    var: Vec<Opp>,
    val: Opp,
}

struct Column(Vec<Row>);
struct Row(Vec<Opp>);

struct Obj(Vec<Entry>);
struct Entry {
    attr: Opp,
    val: Opp,
}

struct Vec1<T> {
    head: Box<T>,
    tail: Vec<T>,
}

struct OppVec<O> {
    head: Box<Opp>,
    tail: Vec1<(O, Opp)>,
}

enum Opp {
    App(OppVec<()>),
    Or(OppVec<()>),
    And(OppVec<()>),
    Equal(OppVec<EqualOpp>),
    Cmp(OppVec<CmpOpp>),
    Add(OppVec<AddOpp>),
    Mul(OppVec<MulOpp>),
    BitOr(OppVec<()>),
    BitAnd(OppVec<()>),
    BitXor(OppVec<()>),
    Shift(OppVec<ShiftOpp>),
    Func(OppVec<()>),
    Term(Box<Term>),
}

enum EqualOpp {
    Equal,
    Unequal,
}

enum CmpOpp {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

enum AddOpp {
    Add,
    Sub,
}

enum MulOpp {
    Mul,
    Div,
    Mod,
}

enum ShiftOpp {
    Left,
    Right,
}

enum Term {
    Num(u64),
    Ident(Rc<str>),
    Expr(Expr),
    Column(Column),
    Obj(Obj),
}
