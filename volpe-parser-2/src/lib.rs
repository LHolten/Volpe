use crate::{file::File, offset::Offset};

extern crate logos;

pub mod file;
mod grammar;
mod lexeme_kind;
pub mod offset;
mod shunting;

#[test]
fn parse() {
    let mut file = File::new();
    file.patch(Offset::char(0), Offset::char(0), "1 1 + ".to_string());
    file.rule();
}
