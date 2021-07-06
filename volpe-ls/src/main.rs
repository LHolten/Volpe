use lsp_server::Connection;

mod capabilities;
mod dispatch;
mod document;
mod handlers;
mod lsp_utils;
mod semantic_tokens;
mod server;
use server::Server;

fn main() {
    let (connection, io_threads) = Connection::stdio();

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

    // TODO Add this back in when doing multithreading.
    // https://github.com/rust-analyzer/rust-analyzer/blob/2c5f905c67bb8e3bfb19efa0de5ab911e63f8e42/crates/rust-analyzer/src/main_loop.rs#L32
    // #[cfg(windows)]
    // unsafe {
    //     use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadPriority};
    //     let thread = GetCurrentThread();
    //     let thread_priority_above_normal = 1;
    //     SetThreadPriority(thread, thread_priority_above_normal);
    // }

    Server::new(connection).run();
    io_threads.join().unwrap();
}
