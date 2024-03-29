#![no_main]
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use volpe_parser_2::{file::File, offset::Offset, ast::ASTBuilder};

#[derive(Debug, Arbitrary)]
struct Edit {
    offset: (u8, u8),
    length: (u8, u8),
    text: [u8; 4],
}

fn to_offset((line, char): (u8, u8)) -> Offset {
    Offset::new(line as usize, char as usize)
}

fuzz_target!(|edits: Vec<Edit>| {
    let mut file = File::default();
    for edit in edits {
        if let Ok(s) = std::str::from_utf8(&edit.text) {
            if !s.chars().all(|c| c.is_ascii()) { return; }
            let offset = to_offset(edit.offset);
            let length = to_offset(edit.length);
            if file.patch(offset, length, s.to_string()).is_err() {
                return;
            }
        } else {
            return;
        }
    }
    if let Ok(syntax) = file.rule().collect() {
        ASTBuilder::default().convert_semicolon(&syntax);
    }
});
