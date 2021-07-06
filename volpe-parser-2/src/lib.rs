extern crate logos;

pub mod error;
pub mod file;
mod grammar;
pub mod lexeme_kind;
pub mod offset;
mod shunting;
pub mod syntax;
pub mod validate;

#[cfg(test)]
mod test {
    use crate::{
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
        dbg!(file.rule());
    }

    #[test]
    fn fuzz_incr_edits_001() -> PatchResult {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), "ȃ".to_string())?;
        file.patch(Offset::char(1), Offset::char(0), "".to_string())?;
        file.rule();
        Ok(())
    }
}
