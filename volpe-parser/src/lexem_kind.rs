use std::ops::BitOr;

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum LexemKind {
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

    #[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
    Ident,

    #[regex("[0-9]+")]
    Num,

    #[error]
    #[regex(r"[\n]")]
    Error,

    End,
}

impl BitOr<LexemKind> for LexemKind {
    type Output = usize;

    fn bitor(self, rhs: LexemKind) -> Self::Output {
        usize::from(self) | rhs
    }
}

impl BitOr<LexemKind> for usize {
    type Output = usize;

    fn bitor(self, rhs: LexemKind) -> Self::Output {
        self | usize::from(rhs)
    }
}

impl From<LexemKind> for usize {
    fn from(val: LexemKind) -> Self {
        1 << val as usize
    }
}

impl Default for LexemKind {
    fn default() -> Self {
        LexemKind::End
    }
}
