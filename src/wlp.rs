use std::cell::Cell;

use typed_arena::Arena;
use volpe_parser::ast::Op;

#[derive(Clone, Copy)]
pub enum WLP<'a> {
    Num(u64),
    Bool(bool),
    Unknown(&'a str),
    Unfinished,
    Op(OpWLP<'a>),
    Ite(IteWLP<'a>),
}

#[derive(Clone, Copy)]
pub struct OpWLP<'a> {
    pub left: &'a Cell<WLP<'a>>,
    pub op: Op,
    pub right: &'a Cell<WLP<'a>>,
}

#[derive(Clone, Copy)]
pub struct IteWLP<'a> {
    pub cond: &'a Cell<WLP<'a>>,
    pub then: &'a Cell<WLP<'a>>,
    pub otherwise: &'a Cell<WLP<'a>>,
}

impl<'a> OpWLP<'a> {
    pub fn new(op: Op, arena: &'a Arena<WLP<'a>>) -> Self {
        Self {
            left: Cell::from_mut(arena.alloc(WLP::Unfinished)),
            op,
            right: Cell::from_mut(arena.alloc(WLP::Unfinished)),
        }
    }
}

impl<'a> IteWLP<'a> {
    pub fn new(arena: &'a Arena<WLP<'a>>) -> Self {
        Self {
            cond: Cell::from_mut(arena.alloc(WLP::Unfinished)),
            then: Cell::from_mut(arena.alloc(WLP::Unfinished)),
            otherwise: Cell::from_mut(arena.alloc(WLP::Unfinished)),
        }
    }
}
