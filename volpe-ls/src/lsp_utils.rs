#![allow(dead_code)]

use lsp_server::Notification;
use lsp_types::{Position, Range};
use volpe_parser_2::offset::Offset;

pub fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}

pub fn to_offset(position: Position) -> Offset {
    Offset::new(position.line as usize, position.character as usize)
}

pub fn to_position(offset: Offset) -> Position {
    Position::new(offset.line as u32, offset.char as u32)
}

pub fn range(range: volpe_parser_2::offset::Range) -> Range {
    Range {
        start: to_position(range.start),
        end: to_position(range.end),
    }
}
