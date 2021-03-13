use std::{cell::Cell, cmp::max, rc::Rc};

use crate::{
    offset::Offset,
    position::{Position, Syntax},
    with_cell::WithInternal,
};

pub type IResult<'t> = Result<Tracker<'t>, ()>;

#[derive(Clone)]
pub struct Tracker<'t> {
    pub pos: Rc<Cell<Position>>,
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
