use lsp_server::{Notification, Request};

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

pub struct RequestHandler<'a> {
    pub request: Option<Request>,
    pub server: &'a mut Server,
}

impl<'a> RequestHandler<'a> {
    pub fn on<R: lsp_types::request::Request>(
        &mut self,
        f: fn(&mut Server, R::Params) -> R::Result,
    ) -> &mut Self {
        let req = match self.request.take() {
            Some(it) => it,
            None => return self,
        };
        let (id, params) = match req.extract::<R::Params>(R::METHOD) {
            Ok(it) => it,
            Err(req) => {
                self.request = Some(req);
                return self;
            }
        };
        // TODO multithread this.
        let result = f(self.server, params);
        // TODO Handle unsuccessful requests.
        self.server
            .send(lsp_server::Response::new_ok(id, &result).into());
        self
    }

    pub fn finish(&mut self) {
        if let Some(req) = &self.request {
            self.server
                .show_error_message(format!("unknown request: {:?}", req));
            let response = lsp_server::Response::new_err(
                req.id.clone(),
                lsp_server::ErrorCode::MethodNotFound as i32,
                "unknown request".to_string(),
            );
            self.server.send(response.into());
        }
    }
}
