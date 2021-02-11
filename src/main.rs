use crate::packrat::Syntax;

extern crate logos;
extern crate typed_arena;
extern crate volpe_parser;
extern crate z3;

mod lexer;
mod packrat;
mod parser;

fn main() {
    let pos = Syntax::default();
    let pos = pos.parse("(x + 1)", 0, 0);
    dbg!(&pos);
    dbg!(pos.text());
}
