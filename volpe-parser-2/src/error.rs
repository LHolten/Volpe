use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PatchError {
    OffsetOutOfRange,
    LengthOutOfRange,
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PatchError::OffsetOutOfRange => "Offset is out of range",
                PatchError::LengthOutOfRange => "Length is out of range",
            }
        )
    }
}

impl Error for PatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
