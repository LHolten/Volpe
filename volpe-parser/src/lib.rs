extern crate logos;

mod combinators;
mod grammar;
pub mod lexeme_kind;
pub mod offset;
pub mod packrat;
pub mod syntax;
mod tracker;

#[cfg(test)]
mod test {

    use crate::offset::Offset;
    use crate::packrat::Parser;

    #[test]
    fn bug() {
        let mut p = Parser::default();
        p.parse("a", Offset::new(0, 0), Offset::new(0, 0));
        // p.padrse("", Offset::new(0, 0), Offset::new(0, 9));
        // p.parse("test", Offset::new(0, 0), Offset::new(0, 0));
    }
}
