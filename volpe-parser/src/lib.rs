#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub volpe);

#[cfg(test)]
mod tests {
    use crate::volpe::BlockParser;

    #[test]
    fn functions() {
        assert!(BlockParser::new().parse("hello world").is_ok());
        assert!(BlockParser::new()
            .parse("[1, 2, 3; 4, 5, 6] 10 (cool thing)")
            .is_ok());
    }
}
