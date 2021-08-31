use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LexemeKind {
    #[regex(";")]
    #[regex(",")]
    Semicolon,

    #[token("=")]
    Assign,

    #[token(".")]
    Abs,

    #[token(":")]
    Case,

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

    // #[regex(r#"#{([^"}]|("([^"\\]|\\.)*"))*}"#)]
    // Wasm,
    #[regex("[A-Z][_a-zA-Z0-9]*")]
    #[regex(r"[#-&*+\-/<=>@\\^|~]+")]
    Unique,

    #[regex("[0-9]+")]
    Num,

    #[regex("//.*")]
    #[error]
    Error,

    App,
}
