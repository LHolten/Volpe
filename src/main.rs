use offset::Offset;

use crate::packrat::Syntax;

extern crate logos;
extern crate typed_arena;
extern crate volpe_parser;
extern crate z3;

mod lexer;
mod offset;
mod packrat;
mod parser;

fn main() {
    let mut pos = Syntax::default();
    pos = pos.parse(
        "b || c 
* 2",
        Offset::char(0),
        Offset::char(0),
    );
    dbg!(pos.text());
    dbg!(&pos);
    pos = pos.parse("", Offset::line(), Offset::char(1));
    dbg!(pos.text());
    dbg!(&pos);
}
