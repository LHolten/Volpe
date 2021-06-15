extern crate logos;

mod error;
pub mod file;
mod grammar;
mod lexeme_kind;
pub mod offset;
mod shunting;

#[cfg(test)]
mod test {
    use crate::{file::File, offset::Offset};

    #[test]
    fn parse() {
        let mut file = File::default();
        file.patch(Offset::char(0), Offset::char(0), "+ (1) 1".to_string());
        dbg!(file.rule());
    }
}
