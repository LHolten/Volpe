use lsp_server::Notification;
use lsp_types;

pub fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}
