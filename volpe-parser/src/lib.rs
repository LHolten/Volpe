#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub volpe);

#[cfg(test)]
mod tests {
    use crate::volpe::ExprParser;

    #[test]
    fn functions() {
        assert!(ExprParser::new().parse("hello world").is_ok());
        assert!(ExprParser::new()
            .parse("[1, 2, 3; 4, 5, 6] 10 (cool thing)")
            .is_ok());
        assert!(ExprParser::new().parse("(a = 1; b = 2; add a b)").is_ok());
    }
}
