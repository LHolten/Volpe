#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub volpe);

#[cfg(test)]
mod tests {
    use crate::volpe::TermParser;

    #[test]
    fn calculator1() {
        assert!(TermParser::new().parse("22").is_ok());
        assert!(TermParser::new().parse("(22)").is_ok());
        assert!(TermParser::new().parse("((((22))))").is_ok());
        assert!(TermParser::new().parse("((22)").is_err());
    }
}
