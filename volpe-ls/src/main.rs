use capabilities::server_capabilities;
use lsp_server::Connection;
use serde_json;

mod capabilities;
mod server;
use server::Server;

fn main() {
    let (connection, _io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(capabilities::server_capabilities()).unwrap();
    let _init_params = connection.initialize(server_capabilities).unwrap();

    Server::new(connection).run();
}
