use lsp_server::Connection;
use lsp_types::ServerCapabilities;
use serde_json;

mod server;
use server::Server;

fn main() {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities::default()).unwrap();
    let init_params = connection.initialize(server_capabilities).unwrap();

    Server::new(connection).run();
}
