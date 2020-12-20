pub mod ast;
#[allow(clippy::all)]
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::parser::{ExprParser, ObjectParser};

    #[test]
    fn functions() {
        assert!(ExprParser::new().parse("hello world").is_ok());
        assert!(ExprParser::new()
            .parse("[1, 2, 3; 4, 5, 6] 10 (cool thing)")
            .is_ok());
        assert!(ExprParser::new().parse("(a = 1; b = 2; add a b)").is_ok());
        assert!(ExprParser::new()
            .parse(
                "my_object = {
                    alpha : something,
                    beta : 3404,
                }; my_object"
            )
            .is_ok());
        assert!(ExprParser::new().parse("a.b.(add a b) 10 20").is_ok());
        assert!(ExprParser::new().parse("{1, 2, 3}").is_ok());
        assert!(ExprParser::new().parse("{1}").is_ok());
        assert!(ExprParser::new().parse("{}").is_ok());
        assert!(ExprParser::new().parse("1 > 2 => {}").is_ok());
    }

    #[test]
    fn complicated_ast() {
        assert!(dbg!(ObjectParser::new().parse(
            "for: iter.func.{
                exec: iter {
                    some: val.(
                        func val;
                        exec
                    ),
                    none: {},
                },
            },
            
            range: from?.to.(
                from >= to => {none};
                val = from;
                from? = from + 1;
                {some, val}
            ),
            
            main: args.(
                total? = 0;
                for (range 10 20) val.(
                    total? = total + val;
                    print total
                ) exec
            ),"
        ))
        .is_ok());
    }
}
