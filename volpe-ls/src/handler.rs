use lsp_server::Notification;
use lsp_types;

use crate::server::Server;

pub struct NotificationHandler<'a> {
    pub notification: Option<Notification>,
    pub server: &'a mut Server,
}

impl<'a> NotificationHandler<'a> {
    pub fn on<N: lsp_types::notification::Notification>(
        &mut self,
        f: fn(&mut Server, N::Params),
    ) -> &mut Self {
        let not = match self.notification.take() {
            Some(it) => it,
            None => return self,
        };
        let params = match not.extract::<N::Params>(N::METHOD) {
            Ok(it) => it,
            Err(not) => {
                self.notification = Some(not);
                return self;
            }
        };
        f(self.server, params);
        self
    }

    pub fn finish(&mut self) {
        if let Some(notif) = &self.notification {
            // notification starting with $/ are optional
            if !notif.method.starts_with("$/") {
                self.server
                    .show_error_message(format!("unhandled notification: {:?}", notif));
            }
        }
    }
}
