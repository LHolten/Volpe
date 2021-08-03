use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum LexemeKind {
    #[regex(r";")]
    Semicolon,

    #[token("=")]
    Assign,

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
    LRoundBracket,

    #[token(")")]
    RRoundBracket,

    #[token("{")]
    LCurlyBracket,

    #[token("}")]
    RCurlyBracket,

    #[regex(r",")]
    Comma,

    #[regex("[_a-z][_a-zA-Z0-9]*")]
    Ident,

    #[regex("[A-Z][_a-zA-Z0-9]*")]
    Const,

    #[regex("[0-9]+")]
    Num,

    #[error]
    #[regex(r"[\n]")]
    Error,

    App,
}
