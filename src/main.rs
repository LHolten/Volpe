use crate::packrat::SharedPosition;

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
    pos.parse("alpha + beta", 0, 0);
    dbg!(pos);
}
