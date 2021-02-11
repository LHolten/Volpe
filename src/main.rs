use crate::packrat::Syntax;

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
    let pos = Syntax::default();
    let pos = pos.parse("2 + 3", 0, 0);
    dbg!(&pos);
    dbg!(pos.text());
    let pos = pos.parse("1 / ", 4, 0);
    dbg!(&pos);
    dbg!(pos.text());
}
