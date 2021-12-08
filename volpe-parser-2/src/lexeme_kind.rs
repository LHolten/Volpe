use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LexemeKind {
    #[regex(";")]
    Semicolon,

    #[token("(")]
    LRoundBracket,

    #[token(")")]
    RRoundBracket,

    #[token("{")]
    LCurlyBracket,

    #[token("}")]
    RCurlyBracket,

    #[token("[")]
    LSquareBracket,

    #[token("]")]
    RSquareBracket,

    #[regex("[_[:alpha:]][_[:alnum:]]*")]
    Ident,

    #[regex(r#"#[[:digit:]]\{([^"}]|("([^"\\]|\\.)*"))*}"#)]
    Wasm,

    #[regex(r"[#-&*+\-/<=>@\\^|~]+")]
    Operator,

    #[regex("[[:digit:]]+")]
    Num,

    #[regex("//.*")]
    #[error]
    Error,

    App,
}
