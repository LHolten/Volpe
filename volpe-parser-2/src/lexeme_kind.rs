use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LexemeKind {
    #[regex(r";")]
    #[regex(r",")]
    Semicolon,

    #[token("=")]
    Assign,

    #[token(".")]
    #[token(":")]
    Abs,

    #[token("(")]
    LRoundBracket,

    #[token(")")]
    RRoundBracket,

    #[token("{")]
    LCurlyBracket,

    #[token("}")]
    RCurlyBracket,

    #[regex("[_a-z][_a-zA-Z0-9]*")]
    Ident,

    #[regex("[A-Z][_a-zA-Z0-9]*")]
    #[regex(r"[#-&*-/<=>@\\^|~]+")]
    Const,

    #[regex("[0-9]+")]
    Num,

    #[error]
    Error,

    App,
}
