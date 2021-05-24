use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    OffsetOutOfRange,
    LengthOutOfRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ParseError::OffsetOutOfRange => "Offset is out of range",
                ParseError::LengthOutOfRange => "Length is out of range",
            }
        )
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
