use lsp_types::{
    HoverProviderCapability, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::Incremental,
        )),
        hover_provider: Some(HoverProviderCapability::from(true)),
        ..Default::default()
    }
}
