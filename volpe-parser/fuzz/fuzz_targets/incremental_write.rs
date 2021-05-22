#![no_main]
use libfuzzer_sys::fuzz_target;
use volpe_parser::{syntax::Lexeme, offset::Offset};

fuzz_target!(|strings: Vec<String>| {
    let mut parser = Lexeme::default();
    for string in &strings {
        parser.parse(string, parser.get_size(), Offset::default());
    }
    assert_eq!(parser.get_text(), strings.join(""));
    same_as_one_shot(parser);
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
