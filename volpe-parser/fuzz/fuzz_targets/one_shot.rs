#![no_main]
use libfuzzer_sys::fuzz_target;
use volpe_parser::{offset::Offset, syntax::Lexeme};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut parser = Lexeme::default();
        parser.parse(s, Offset::default(), Offset::default());
        assert_eq!(s, parser.get_text());
    }
});
