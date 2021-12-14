use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Offset {
    pub line: usize,
    pub char: usize,
}

impl Offset {
    pub fn new(line: usize, char: usize) -> Self {
        Self { line, char }
    }
    pub fn line() -> Self {
        Self { line: 1, char: 0 }
    }
    pub fn char(char: usize) -> Self {
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

#[derive(Clone, Copy, Default, Debug)]
pub struct Range<'a> {
    pub start: Offset,
    pub end: Offset,
    pub text: &'a str,
}

impl<'a> Range<'a> {
    pub fn raw_inner(self) -> Self {
        Self {
            start: self.start + Offset::char(2),
            end: self.end - Offset::char(1),
            text: &self.text[2..self.text.len() - 1],
        }
    }

    pub fn length(self) -> Offset {
        self.end - self.start
    }
}
