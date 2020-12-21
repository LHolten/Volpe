pub mod ast;
#[allow(clippy::all)]
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::parser::{ExprParser, ObjectParser};

    macro_rules! test_expr {
        ($s:literal) => {
            assert!(ExprParser::new().parse($s).is_ok())
        };
    }

    #[test]
    fn functions() {
        test_expr!("hello world");
        test_expr!("[1, 2, 3; 4, 5, 6] 10 (cool thing)");
        test_expr!("(a = 1; b = 2; add a b)");
        test_expr!(
            "my_object = {
                alpha : something,
                beta : 3404,
            }; my_object"
        );
        test_expr!("a.b.(add a b) 10 20");
        test_expr!("{1, 2, 3}");
        test_expr!("{1}");
        test_expr!("{}");
        test_expr!("1 > 2 => {}");
        test_expr!("1 /* /* wow */ cool */ > 2 // hello");
    }

    #[test]
    fn complicated_ast() {
        assert!(dbg!(ObjectParser::new().parse(
            "// for loop implementation
            for: iter.func.{
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

            /* this main closure will be called
              when we execute the program */
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
