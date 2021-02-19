use lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::Incremental,
        )),
        ..Default::default()
    }
}
