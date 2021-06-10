use crate::offset::Offset;

pub struct File {
    pub lines: Vec<String>,
}

impl File {
    pub fn patch(&mut self, offset: Offset, length: Offset, mut text: String) {
        let end = offset + length;
        text.push_str(&self.lines[end.line][end.char..]);
        let mut text_lines = text.lines();

        let first = &mut self.lines[offset.line];
        first.truncate(offset.char);
        first.push_str(text_lines.next().unwrap());

        self.lines.splice(
            offset.line + 1..end.line + 1,
            text_lines.map(str::to_string),
        );
    }
}

impl Default for File {
    fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
        }
    }
}
