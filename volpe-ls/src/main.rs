use lsp_server::Connection;
use lsp_types;
use serde_json;

extern crate volpe_parser;

mod capabilities;
mod document;
mod handler;
mod lsp_utils;
mod server;
use server::Server;

fn main() {
    let (connection, _io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start().unwrap();
    let _init_params: lsp_types::InitializeParams =
        serde_json::from_value(initialize_params).unwrap();

    let initialize_result = lsp_types::InitializeResult {
        capabilities: capabilities::server_capabilities(),
        server_info: None,
        offset_encoding: Some("utf-8".to_string()), // we ONLY support utf-8
    };
    let initialize_result = serde_json::to_value(initialize_result).unwrap();

    connection
        .initialize_finish(initialize_id, initialize_result)
        .unwrap();

    Server::new(connection).run();
}
