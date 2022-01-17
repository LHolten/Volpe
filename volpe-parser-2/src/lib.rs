extern crate logos;
extern crate string_interner;
extern crate void;

mod display;
pub mod error;
pub mod eval;
pub mod file;
mod grammar;
pub mod lexeme_kind;
pub mod offset;
mod shunting;
pub mod simple;
pub mod syntax;
pub mod validate;

#[cfg(test)]
mod test {
    use crate::{
        eval::Evaluator,
        file::{File, PatchResult},
        offset::Offset,
    };

    #[test]
    fn parse() {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "".to_string())
            .unwrap();
        file.rule();

        let mut file = File::default();
        file.patch(Offset::char(0), Offset::char(0), "+ (1) 1".to_string())
            .unwrap();
        file.rule();
    }

    #[test]
    fn manual_001() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "(".to_string())?;
        file.rule();
        Ok(())
    }

    #[test]
    fn fuzz_incr_edits_001() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "ȃ".to_string())?;
        assert!(file
            .patch(Offset::char(1), Offset::char(0), "".to_string())
            .is_err());
        file.rule();
        Ok(())
    }

    #[test]
    fn fuzz_incr_edits_002() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "%=>".to_string())?;
        file.rule();
        Ok(())
    }

    #[test]
    fn fuzz_incr_edits_003() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), ")".to_string())?;
        file.rule();
        Ok(())
    }

    #[test]
    fn fuzz_incr_edits_004() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "]".to_string())?;
        file.rule();
        Ok(())
    }

    #[test]
    fn fuzz_incr_edits_005() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "[1]".to_string())?;
        if let Ok(syntax) = file.rule().collect() {
            syntax.convert();
        }
        Ok(())
    }

    #[test]
    fn test_eval() -> PatchResult {
        let mut file = File::default();
        file.patch(
            Offset::default(),
            Offset::default(),
            "
            [i32] ( [n]
                #{(i32.const };
                n;
                #{)}
            );
            [add] (#{(i32.add)});

            add i32(1) i32(2)
            "
            .to_string(),
        )?;
        let syntax = dbg!(file.rule().collect().unwrap());
        let ast = dbg!(syntax.convert());
        dbg!(Evaluator::eval(ast).unwrap());
        Ok(())
    }
}
