use std::rc::Rc;

use crate::{
    offset::Offset,
    syntax::{Lexeme, Syntax},
};

pub trait TFunc {
    fn parse(t: TInput) -> TResult;
}
pub type TResult = Result<TInput, Tracker>;

#[derive(Default)]
pub struct Tracker {
    pub children: Vec<Syntax>,
    pub length: Offset,
}

pub struct TInput {
    pub lexeme: Rc<Lexeme>,
    pub offset: Offset,
    pub tracker: Tracker,
}

impl From<TResult> for Tracker {
    fn from(result: TResult) -> Self {
        match result {
            Ok(input) => input.tracker,
            Err(tracker) => tracker,
        }
    }
}
