use crate::{offset::Offset, syntax::Lexeme};

pub trait TFunc {
    fn parse(t: TInput) -> Result<TInput, TError>;
}

pub type TResult<'a> = Result<TInput<'a>, TError>;

pub struct TInput<'a> {
    pub lexeme: &'a mut Option<Box<Lexeme>>,
    pub length: Offset, // the length of the text consumed by the current rule
    pub error: TError,
}

#[allow(clippy::vec_box)]
pub struct TError {
    pub remaining: Vec<Box<Lexeme>>,
    pub sensitive_length: Offset, // the length of text that triggers reparse if changed
}
