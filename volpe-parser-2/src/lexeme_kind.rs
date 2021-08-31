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

    #[regex("[_[:lower:]][_[:alnum:]]*")]
    Ident,

    #[regex(r#"#[[:digit:]]\{([^"}]|("([^"\\]|\\.)*"))*}"#)]
    Wasm,

    #[regex("[[:upper:]][_[:alnum:]]*")]
    #[regex(r"[#-&*+\-/<=>@\\^|~]+")]
    Unique,

    #[regex("[[:digit:]]+")]
    Num,

    #[regex("//.*")]
    #[error]
    Error,

    App,
}
