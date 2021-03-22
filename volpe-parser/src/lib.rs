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

    macro_rules! test_expr {
        ($s:literal) => {
            let mut parser = Parser::default();
            parser.parse($s, Offset::default(), Offset::default());
        };
    }

    #[test]
    fn thing() {
        test_expr!("=");
        test_expr!("=>");
        test_expr!("(");
        test_expr!(")");
        test_expr!("()");
        test_expr!("");
        test_expr!("=>");
    }

}
