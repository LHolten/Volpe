use std::collections::HashMap;

// use crossbeam::channel::{Receiver, Sender};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{self, notification::ShowMessage, MessageType, ShowMessageParams};

use crate::document::Document;
use crate::handler::{NotificationHandler, RequestHandler};

pub struct Server {
    connection: Connection,
    documents: HashMap<String, Document>,
}

// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/rust-analyzer/src/global_state.rs
impl Server {
    pub fn new(connection: Connection) -> Server {
        Server {
            connection,
            documents: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        self.show_info_message("Started :)".to_string());

        while let Ok(message) = self.connection.receiver.recv() {
            // self.show_info_message(format!("{:?}", message));
            match message {
                Message::Request(request) => {
                    self.show_info_message(format!(
                        "received a request! method: {} (id: {})",
                        request.method, request.id
                    ));
                    if self.connection.handle_shutdown(&request).unwrap() {
                        break;
                    };
                    self.handle_request(request);
                }
                Message::Response(response) => {
                    // TODO Once we start sending requests we should handle the response here.
                    self.show_info_message(format!("received a response! (id: {})", response.id));
                }
                Message::Notification(notificaiton) => {
                    self.show_info_message(format!(
                        "received a notification! method: {}",
                        notificaiton.method
                    ));
                    self.handle_notification(notificaiton);
                }
            }
        }
    }

    fn handle_notification(&mut self, notification: Notification) {
        NotificationHandler {
            notification: Some(notification),
            server: self,
        }
        .on::<lsp_types::notification::DidOpenTextDocument>(|this, params| {
            this.documents
                .insert(params.text_document.uri.to_string(), Document::new(&params));
        })
        .on::<lsp_types::notification::DidChangeTextDocument>(|this, params| {
            let uri = params.text_document.uri.to_string();
            match this.documents.get_mut(&uri) {
                Some(doc) => doc.update(&params),
                None => this.show_error_message(format!("{} was not found in documents", uri)),
            }
        })
        .on::<lsp_types::notification::DidSaveTextDocument>(|_this, _params| {})
        .on::<lsp_types::notification::DidCloseTextDocument>(|_this, _params| {})
        .finish();
    }

    fn handle_request(&mut self, request: Request) {
        RequestHandler {
            request: Some(request),
            server: self,
        }
        .on::<lsp_types::request::HoverRequest>(|this, params| {
            let potential_doc = this.documents.get(
                &params
                    .text_document_position_params
                    .text_document
                    .uri
                    .to_string(),
            );
            match potential_doc {
                Some(doc) => Some(lsp_types::Hover {
                    contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                        kind: lsp_types::MarkupKind::PlainText,
                        value: doc.get_info(),
                    }),
                    range: None,
                }),
                None => None,
            }
        })
        .on::<lsp_types::request::SemanticTokensFullRequest>(|this, params| {
            let potential_doc = this
                .documents
                .get_mut(&params.text_document.uri.to_string());
            match potential_doc {
                Some(doc) => {
                    let tokens = doc.get_semantic_tokens();
                    Some(lsp_types::SemanticTokensResult::Tokens(tokens))
                }
                None => None,
            }
        })
        .finish();
    }
}

impl Server {
    pub fn send_notification<N: lsp_types::notification::Notification>(
        &mut self,
        params: N::Params,
    ) {
        let not = Notification::new(N::METHOD.to_string(), params);
        self.send(not.into());
    }

    // pub fn send_request<R: lsp_types::request::Request>(
    //     &mut self,
    //     params: R::Params,
    //     result: R::Result,
    // ) {
    //     // TODO Use ReqQueue to assign id.
    //     let req = Request::new(0, R::METHOD.to_string(), params);
    //     self.send(req.into())
    // }

    pub fn send(&mut self, message: Message) {
        self.connection.sender.send(message).unwrap()
    }
}

impl Server {
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
