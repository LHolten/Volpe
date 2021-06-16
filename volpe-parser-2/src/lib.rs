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
        file.patch(Offset::default(), Offset::default(), "".to_string())
            .unwrap();
        file.rule();

        let mut file = File::default();
        file.patch(Offset::char(0), Offset::char(0), "+ (1) 1".to_string())
            .unwrap();
        dbg!(file.rule());
    }
}
