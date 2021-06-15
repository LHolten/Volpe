#![no_main]
use libfuzzer_sys::fuzz_target;
use volpe_parser_2::{file::File, offset::Offset};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), s.to_string()).unwrap();
        file.rule();
    }
});
