extern crate logos;

mod combinators;
pub mod lexeme_kind;
pub mod offset;
pub mod packrat;
mod parser;
pub mod syntax;
mod tracker;

#[cfg(test)]
mod test {

    use crate::offset::Offset;
    use crate::packrat::Parser;

    #[test]
    fn bug() {
        let mut p = Parser::default();
        p.parse("a + b + c", Offset::new(0, 0), Offset::new(0, 0));
        p.parse("", Offset::new(0, 0), Offset::new(0, 9));
        p.parse("test", Offset::new(0, 0), Offset::new(0, 0));
    }
}
