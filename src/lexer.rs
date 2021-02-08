use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Lexem {
    #[token("=")]
    Assign,

    #[token(":=")]
    MultiAssign,

    #[token("=>")]
    Ite,

    #[token(".")]
    Func,

    #[token("||")]
    Or,

    #[token("&&")]
    And,

    #[token("==")]
    Equals,

    #[token("!=")]
    UnEquals,

    #[token("<")]
    Less,

    #[token(">")]
    Greater,

    #[token("<=")]
    LessEqual,

    #[token(">=")]
    GreaterEqual,

    #[token("|")]
    BitOr,

    #[token("&")]
    BitAnd,

    #[token("^")]
    BitXor,

    #[token("<<")]
    BitShl,

    #[token(">>")]
    BitShr,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    #[token("%")]
    Mod,

    #[token("(")]
    LBrace,

    #[token(")")]
    RBrace,

    #[regex(r"[;\n]")]
    NewLine,

    #[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
    Ident,

    #[regex("[0-9]+")]
    Num,

    #[error]
    #[regex(r"[ \t]+")]
    Error,
}
