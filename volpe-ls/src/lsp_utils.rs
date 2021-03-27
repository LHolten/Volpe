use lsp_server::Notification;

pub fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}
