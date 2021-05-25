use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
pub use volpe_parser::{error::ParseError, offset::Offset, syntax::Lexeme};

#[macro_export]
macro_rules! test_expr {
    ($s:literal) => {{
        let mut parser = Lexeme::default();
        parser
            .parse($s, Offset::default(), Offset::default())
            .unwrap();
        parser
    }};
}

#[macro_export]
macro_rules! test_edits {
    ($([$s:literal, ($offset_line:literal, $offset_char:literal), ($length_line:literal, $length_char:literal)])*) => {
        {
            let mut parser = Lexeme::default();
            $(parser.parse($s, Offset::new($offset_line, $offset_char), Offset::new($length_line, $length_char))?;)*
            Ok(())
        }
    };
}

#[allow(clippy::mutex_atomic, dead_code)]
pub fn with_timeout<F: 'static + FnOnce() + Send>(f: F, duration: Duration) {
    let finished = Arc::new((Mutex::new(false), Condvar::new()));
    thread::spawn({
        let finished = Arc::clone(&finished);
        move || {
            f();
            let mut guard = finished.0.lock().unwrap();
            *guard = true;
            finished.1.notify_one();
        }
    });

    let timed_out = finished
        .1
        .wait_timeout_while(finished.0.lock().unwrap(), duration, |&mut finished| {
            !finished
        })
        .unwrap()
        .1
        .timed_out();

    assert!(!timed_out);
}
