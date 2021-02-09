use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};
use lsp_server::{Message, Notification, Request, Response};
use lsp_types::{self, MessageType, ShowMessageParams, notification::{Exit, Notification as _, ShowMessage}, request::{self, Request as _, Shutdown}};

pub struct Server {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/rust-analyzer/src/global_state.rs
impl Server {
    pub fn new(sender: Sender<Message>, receiver: Receiver<Message>) -> Server {
        Server { sender, receiver }
    }

    pub fn run(&mut self) {
        self.show_info_message("Started :)".to_string());
        
        while let Ok(message) = self.receiver.recv() {
            self.show_info_message(format!("{:?}", message));
            match message {
                Message::Request(request) => {
                    self.show_info_message(format!("received a request! (id: {})", request.id));
                    if &request.method == Shutdown::METHOD {
                        self.show_warning_message("it's a shutdown request".to_string());
                        self.send(Response::new_ok(request.id, serde_json::Value::Null).into());
                        break;
                    }
                },
                Message::Response(response) => {
                    self.show_info_message(format!("received a response! (id: {})", response.id));
                },
                Message::Notification(notificaiton) => {
                    self.show_info_message("received a notification!".to_string());
                }
            }
        }

        // try to recv exit notification
        // TODO Figure out how this works
        let message = self.receiver.recv_timeout(Duration::from_secs(1));
        if let Ok(message) = message {
            self.show_info_message(format!("after shutdown message {:?}", message));
        };
    }

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

    // pub fn send_response<O, E>(
    //     &mut self,
    //     result: Result<O, E>,
    //     id: lsp_server::RequestId
    // ) {
    //     let req = match result {
    //         Ok(ok) => Response::new_ok(id, serde_json::Value::Null),
    //         Err(err) => Response::new_err(id, ???, String::from(err))
    //     };
    //     self.send(req.into())
    // }

    fn send(&mut self, message: Message) {
        self.sender.send(message).unwrap()
    }
}

impl Server {
    pub fn show_message(&mut self, typ: MessageType, message: String) {
        self.send_notification::<ShowMessage>(ShowMessageParams {typ, message})
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
