#![no_main]
use libfuzzer_sys::{fuzz_target, arbitrary::Arbitrary};
use volpe_parser::{syntax::Lexeme, offset::Offset};

#[derive(Debug, Arbitrary)]
struct Insert {
    offset: (u32, u32),
    text: String,
}

fuzz_target!(|inserts: Vec<Insert>| {
    let mut parser = Lexeme::default();
    for insert in inserts {
        let offset = Offset::new(insert.offset.0, insert.offset.1);
        if offset > parser.get_size() { break; }
        parser.parse(&insert.text, offset, Offset::default());
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
