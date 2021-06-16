use crate::error::PatchError;
use crate::offset::Offset;

pub struct File {
    pub lines: Vec<String>,
}

type PatchResult = Result<(), PatchError>;

impl File {
    pub fn patch(&mut self, offset: Offset, length: Offset, mut text: String) -> PatchResult {
        let end = offset + length;
        if offset.line >= self.lines.len() || offset.char > self.lines[offset.line].len() {
            return Err(PatchError::OffsetOutOfRange);
        }
        if end.line >= self.lines.len() || end.char > self.lines[end.line].len() {
            return Err(PatchError::LengthOutOfRange);
        }

        text.push_str(&self.lines[end.line][end.char..]);
        let mut text_lines = text.split('\n');

        let first = &mut self.lines[offset.line];
        first.truncate(offset.char);
        first.push_str(text_lines.next().unwrap());

        self.lines.splice(
            offset.line + 1..end.line + 1,
            text_lines.map(str::to_string),
        );

        Ok(())
    }
}

impl Default for File {
    fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
        }
    }
}
