use crate::semantic_tokens::{SUPPORTED_MODIFIERS, SUPPORTED_TYPES};
use lsp_types::*;

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::Incremental,
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: SUPPORTED_TYPES.to_vec(),
                    token_modifiers: SUPPORTED_MODIFIERS.to_vec(),
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: Some(true),
                ..Default::default()
            }
            .into(),
        ),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}
