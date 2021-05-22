#![no_main]
use libfuzzer_sys::fuzz_target;
use volpe_parser::{syntax::Lexeme, offset::Offset};

fuzz_target!(|strings: Vec<&str>| {
    let mut parser = Lexeme::default();
    let mut length = Offset::default();
    for &s in &strings {
        parser.parse(s, length, Offset::default());
        length += Offset::measure(s);
    }
    assert_eq!(strings.join(""), parser.get_text())
});
