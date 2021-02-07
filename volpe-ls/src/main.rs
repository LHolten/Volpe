use crossbeam::channel::Sender;

use lsp_server::{self, Connection};
use lsp_types::{MessageType, ServerCapabilities, ShowMessageParams, notification::{Notification, ShowMessage}};
use serde_json;

fn main() {
    let (mut connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities::default()).unwrap();
    let init_params = connection.initialize(server_capabilities).unwrap();

    show_message(&mut connection.sender, ShowMessageParams {typ: MessageType::Info, message: "hello world".to_string()});

    loop {};
}

fn show_message(
    sender: &mut Sender<lsp_server::Message>,
    params: ShowMessageParams,
) {
    let not = lsp_server::Notification::new(ShowMessage::METHOD.to_string(), params);
    sender.send(not.into()).unwrap();
}