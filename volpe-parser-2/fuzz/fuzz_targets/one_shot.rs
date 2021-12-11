#![no_main]
use libfuzzer_sys::fuzz_target;
use volpe_parser_2::{file::File, offset::Offset, ast::ASTBuilder};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut file = File::default();
        if file.patch(Offset::default(), Offset::default(), s.to_string()).is_ok() {
            if let Ok(syntax) = file.rule().collect() {
                ASTBuilder::default().convert_semicolon(&syntax);
            }
        }
    }
});
