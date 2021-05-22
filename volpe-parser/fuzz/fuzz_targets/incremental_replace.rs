#![no_main]
use libfuzzer_sys::{fuzz_target, arbitrary::Arbitrary};
use volpe_parser::{syntax::Lexeme, offset::Offset};

#[derive(Debug, Arbitrary)]
struct Replace {
    offset: (u32, u32),
    length: (u32, u32),
    text: String,
}

fuzz_target!(|inserts: Vec<Replace>| {
    let mut parser = Lexeme::default();
    for insert in inserts {
        let size = parser.get_size();
        let offset = Offset::new(insert.offset.0, insert.offset.1);
        let length = Offset::new(insert.length.0, insert.length.1);
        if offset > size { break; }
        if offset + length > size { break; }
        parser.parse(&insert.text, offset, length);
    }
    same_as_one_shot(parser)
});

/// Check that the parse is the same as one-shot.
fn same_as_one_shot(parser: Lexeme) {
    let mut one_shot = Lexeme::default();
    one_shot.parse(&parser.get_text(), Offset::default(), Offset::default());
    assert_eq!(
        format!("{}", parser),
        format!("{}", one_shot),
    );
}
