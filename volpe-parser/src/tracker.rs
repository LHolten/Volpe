use std::{cell::Cell, cmp::max, rc::Rc};

use crate::{
    offset::Offset,
    syntax::{Lexem, Syntax},
    with_internal::WithInternal,
};

pub type IResult<'t> = Result<Tracker<'t>, ()>;

#[derive(Clone)]
pub struct Tracker<'t> {
    pub pos: Rc<Cell<Lexem>>,
    pub offset: Offset,
    pub children: &'t Cell<Vec<Syntax>>,
    pub length: &'t Cell<Offset>,
}

impl<'t> Tracker<'t> {
    pub fn add_child(&self, child: Syntax) {
        self.children.with(|children| children.push(child));
    }

    pub fn update_length(&self, length: Offset) {
        self.length
            .set(max(self.offset + length, self.length.get()));
    }
}
