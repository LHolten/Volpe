use std::ops::BitOr;

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
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

    #[token("{")]
    LCurlyBrace,

    #[token("}")]
    RCurlyBrace,

    #[regex(r"[;\n]")]
    NewLine,

    #[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
    Ident,

    #[regex("[0-9]+")]
    Num,

    #[error]
    #[regex(r"[ \t]+")]
    Error,

    End,
}

impl BitOr<Lexem> for Lexem {
    type Output = usize;

    fn bitor(self, rhs: Lexem) -> Self::Output {
        usize::from(self) | rhs
    }
}

impl BitOr<Lexem> for usize {
    type Output = usize;

    fn bitor(self, rhs: Lexem) -> Self::Output {
        self | usize::from(rhs)
    }
}

impl From<Lexem> for usize {
    fn from(val: Lexem) -> Self {
        1 << val as usize
    }
}

impl Default for Lexem {
    fn default() -> Self {
        Lexem::End
    }
}
