use packrat::RuleRef;

use crate::{
    packrat::{RuleKind, SharedPosition},
    parser::expr,
};

extern crate logos;
extern crate typed_arena;
extern crate volpe_parser;
extern crate z3;

mod core;
mod lexer;
mod packrat;
mod parser;
mod state;
mod tree;

fn main() {
    let pos = SharedPosition::new();
    pos.patch(None, "hello world", 0, 0).unwrap_err();
    assert!(pos.parse(expr).is_ok());
    dbg!(RuleRef(Some(RuleKind::Expr), pos));
}
