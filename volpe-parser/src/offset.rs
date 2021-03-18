use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Offset {
    pub line: u32,
    pub char: u32,
}

impl Offset {
    pub fn new(line: u32, char: u32) -> Self {
        Self { line, char }
    }
    pub fn line() -> Self {
        Self { line: 1, char: 0 }
    }
    pub fn char(char: u32) -> Self {
        Self { line: 0, char }
    }
}

impl AddAssign for Offset {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Add for Offset {
    type Output = Offset;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line + rhs.line,
            char: if rhs.line == 0 {
                self.char + rhs.char
            } else {
                rhs.char
            },
        }
    }
}

impl SubAssign for Offset {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Sub for Offset {
    type Output = Offset;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line - rhs.line,
            char: if self.line == rhs.line {
                self.char - rhs.char
            } else {
                self.char
            },
        }
    }
}
