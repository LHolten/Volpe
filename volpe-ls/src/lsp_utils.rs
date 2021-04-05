use lsp_server::Notification;
use lsp_types::Position;
use volpe_parser::offset::Offset;

#[allow(dead_code)]
pub fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}

pub fn to_offset(position: Position) -> Offset {
    Offset::new(position.line, position.character)
}

pub fn to_position(offset: Offset) -> Position {
    Position::new(offset.line, offset.char)
}
