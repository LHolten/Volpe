use lsp_server::{Notification, Request};

use crate::handlers::RequestResult;
use crate::server::Server;

pub struct NotificationDispatcher<'a> {
    pub notification: Option<Notification>,
    pub server: &'a mut Server,
}

impl<'a> NotificationDispatcher<'a> {
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
        if let Some(notification) = &self.notification {
            // notification starting with $/ are optional
            if !notification.method.starts_with("$/") {
                self.server
                    .show_error_message(format!("unhandled notification: {:?}", notification));
            }
        }
    }
}

pub struct RequestDispatcher<'a> {
    pub request: Option<Request>,
    pub server: &'a mut Server,
}

impl<'a> RequestDispatcher<'a> {
    pub fn on<R: lsp_types::request::Request>(
        &mut self,
        f: fn(&mut Server, R::Params) -> RequestResult<R::Result>,
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
        self.server.send_response::<R>(id, result);
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
