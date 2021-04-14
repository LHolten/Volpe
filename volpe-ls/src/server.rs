use std::collections::HashMap;

use lsp_server::{Connection, Message, Notification, Request};
use lsp_types::{self, notification::ShowMessage, MessageType, ShowMessageParams};

use crate::dispatch::{NotificationDispatcher, RequestDispatcher};
use crate::document::Document;
use crate::handlers::{self, RequestResult};

pub struct Server {
    connection: Connection,
    pub documents: HashMap<String, Document>,
}

impl Server {
    pub fn new(connection: Connection) -> Server {
        Server {
            connection,
            documents: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        self.show_info_message("Started Volpe Language Server :)".to_string());
        while let Ok(message) = self.connection.receiver.recv() {
            match message {
                Message::Request(request) => {
                    if self.connection.handle_shutdown(&request).unwrap() {
                        break;
                    };
                    self.on_request(request);
                }
                Message::Response(response) => {
                    self.show_warning_message(format!(
                        "received a response??? (id: {})",
                        response.id
                    ));
                    // TOTO self.on_response
                }
                Message::Notification(notification) => {
                    self.on_notification(notification);
                }
            }
        }
    }

    fn on_notification(&mut self, notification: Notification) {
        NotificationDispatcher {
            notification: Some(notification),
            server: self,
        }
        .on::<lsp_types::notification::DidOpenTextDocument>(
            handlers::did_open_text_document_notification,
        )
        .on::<lsp_types::notification::DidChangeTextDocument>(
            handlers::did_change_text_document_notification,
        )
        .on::<lsp_types::notification::DidSaveTextDocument>(|_this, _params| {})
        .on::<lsp_types::notification::DidCloseTextDocument>(|_this, _params| {})
        .finish();
    }

    fn on_request(&mut self, request: Request) {
        #[rustfmt::skip]
        RequestDispatcher {
            request: Some(request),
            server: self,
        }
        .on::<lsp_types::request::HoverRequest>(handlers::hover)
        .on::<lsp_types::request::SemanticTokensFullRequest>(handlers::semantic_tokens_full)
        .on::<lsp_types::request::SemanticTokensRangeRequest>(handlers::semantic_tokens_range)
        .on::<lsp_types::request::GotoDefinition>(handlers::goto_definition)
        .finish();
    }
}

#[allow(dead_code)]
impl Server {
    pub fn send(&mut self, message: Message) {
        self.connection.sender.send(message).unwrap()
    }

    pub fn send_notification<N: lsp_types::notification::Notification>(
        &mut self,
        params: N::Params,
    ) {
        let not = Notification::new(N::METHOD.to_string(), params);
        self.send(not.into());
    }

    pub fn send_response<R: lsp_types::request::Request>(
        &mut self,
        id: lsp_server::RequestId,
        result: RequestResult<R::Result>,
    ) {
        let res = match result {
            Ok(result) => lsp_server::Response::new_ok(id, result),
            Err((err_code, err_msg)) => lsp_server::Response::new_err(id, err_code as i32, err_msg),
        };
        self.send(res.into());
    }

    // TODO send_request (not needed rn because we never send requests)

    pub fn show_message(&mut self, typ: MessageType, message: String) {
        self.send_notification::<ShowMessage>(ShowMessageParams { typ, message })
    }

    pub fn show_error_message(&mut self, message: String) {
        self.show_message(MessageType::Error, message)
    }

    pub fn show_warning_message(&mut self, message: String) {
        self.show_message(MessageType::Warning, message)
    }

    pub fn show_info_message(&mut self, message: String) {
        self.show_message(MessageType::Info, message)
    }

    pub fn show_log_message(&mut self, message: String) {
        self.show_message(MessageType::Log, message)
    }
}
