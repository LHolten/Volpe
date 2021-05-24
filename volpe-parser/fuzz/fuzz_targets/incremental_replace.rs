#![no_main]
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use volpe_parser::{offset::Offset, syntax::Lexeme};

#[derive(Debug, Arbitrary)]
struct Edit {
    offset: (u8, u8),
    length: (u8, u8),
    text: [u8; 4],
}

fn to_offset((line, char): (u8, u8)) -> Offset {
    Offset::new(line as u32, char as u32)
}

fuzz_target!(|edits: Vec<Edit>| {
    let mut parser = Lexeme::default();
    for edit in edits {
        if let Ok(string) = std::str::from_utf8(&edit.text) {
            let offset = to_offset(edit.offset);
            let length = to_offset(edit.length);
            if let None = parser.parse(string, offset, length) {
                return;
            }
        } else {
            return;
        }
    }
    // same_as_one_shot(parser)
});

/// Check that the parse is the same as one-shot.
#[allow(dead_code)]
fn same_as_one_shot(parser: Lexeme) {
    let mut one_shot = Lexeme::default();
    one_shot.parse(&parser.get_text(), Offset::default(), Offset::default()).unwrap();
    assert_eq!(format!("{}", parser), format!("{}", one_shot),);
}
