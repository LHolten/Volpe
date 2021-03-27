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
            {
                let mut parser = Parser::default();
                parser.parse($s, Offset::default(), Offset::default());
                parser
            }
        };
    }

    // Use this if you want to see how the parse tree changes with each edit.
    macro_rules! examine_parser {
        ( $i:ident, $( $e:expr )* ) => {
            $(
                $e;
                println!("{}", $i);
            )*
        }
    }

    #[test]
    fn basic_symbols() {
        test_expr!("");
        test_expr!("a");
        test_expr!("0");
        test_expr!("+");
        test_expr!("-");
        test_expr!("*");
        test_expr!("/");
        test_expr!("%");
        test_expr!("=");
        test_expr!(":=");
        test_expr!(":");
        test_expr!("=>");
        test_expr!("(");
        test_expr!(")");
        test_expr!("()");
        test_expr!("{");
        test_expr!("}");
        test_expr!("{}");
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", test_expr!("1 => 2 / 3; 4")),
            "Expr\n  (Num, \"1 \")\n  (Ite, \"=> \")\n  Op3\n    (Num, \"2 \")\n    (Div, \"/ \")\n    (Num, \"3\")\n  (Semicolon, \"; \")\n  (Num, \"4\")\n"
        )
    }

    #[test]
    fn incremental_parsing() {
        let mut parser = Parser::default();
        parser.parse("a", Offset::default(), Offset::default());
        parser.parse(" ", Offset::char(1), Offset::char(0));
        parser.parse("+", Offset::char(2), Offset::char(0));
        parser.parse(" ", Offset::char(3), Offset::char(0));
        parser.parse("b", Offset::char(3), Offset::char(0));
        parser.parse("(", Offset::char(0), Offset::char(0));
        parser.parse(" ", Offset::char(2), Offset::char(2));
        parser.parse(")", Offset::char(4), Offset::char(0));
    }    

    // TODO add some more complicated tests.
}
