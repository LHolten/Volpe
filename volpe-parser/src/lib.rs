extern crate logos;

pub mod ast;
mod combinators;
mod lexem_kind;
mod offset;
pub mod packrat;
mod parser;
mod syntax;
mod tracker;
mod with_internal;

#[cfg(test)]
mod test {

    use crate::offset::Offset;
    use crate::syntax::Syntax;

    #[test]
    fn bug() {
        let mut p = Syntax::default();
        p = p.parse("a + b + c", Offset::new(0, 0), Offset::new(0, 0));
        p = p.parse("", Offset::new(0, 0), Offset::new(0, 9));
        println!("{:#?}", p);
    }
}
