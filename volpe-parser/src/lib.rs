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

    macro_rules! examine_parser {
        ( $i:ident, $( $e:expr )* ) => {
            $(
                $e;
                println!("{}", $i);
            )*
        }
    }

    #[test]
    fn thing() {
        test_expr!("=");
        test_expr!("=>");
        test_expr!("(");
        test_expr!(")");
        test_expr!("()");
        test_expr!("");
    }

    #[test]
    fn more_things() {
        let mut parser = Parser::default();
        examine_parser!(
            parser,
            parser.parse("a", Offset::default(), Offset::default())
            parser.parse(" ", Offset::char(1), Offset::char(0))
            parser.parse("<", Offset::char(2), Offset::char(0))
        );
    }
}
