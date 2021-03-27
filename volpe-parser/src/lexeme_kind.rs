use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum LexemeKind {
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

    #[token("{")]
    LCurlyBrace,

    #[token("}")]
    RCurlyBrace,

    #[regex(r";")]
    Semicolon,

    #[regex(r":")]
    Colon,

    #[regex(r",")]
    Comma,

    #[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
    Ident,

    #[regex("[0-9]+")]
    Num,

    #[error]
    #[regex(r"[\n]")]
    Error,

    Start,
}

impl LexemeKind {
    pub const fn mask(self) -> usize {
        1 << self as usize
    }
}

impl Default for LexemeKind {
    fn default() -> Self {
        LexemeKind::Start
    }
}
