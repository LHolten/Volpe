use lsp_types::*;

use crate::server::Server;
use crate::document::Document;

//
// Notifications
//

pub fn did_open_text_document_notification(this: &mut Server, params: DidOpenTextDocumentParams) {
    let doc = Document::new(&params);
    let path_buf = params
        .text_document
        .uri
        .to_file_path()
        .unwrap()
        .parent()
        .unwrap()
        .join("volpe_parse_tree.txt");
    if let Err(err_msg) = doc.write_tree_to_file(path_buf.as_path()) {
        this.show_error_message(err_msg)
    };
    this.documents
        .insert(params.text_document.uri.to_string(), doc);
}

pub fn did_change_text_document_notification(this: &mut Server, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri.to_string();
    match this.documents.get_mut(&uri) {
        Some(doc) => {
            doc.update(&params);
            let path_buf = params
                .text_document
                .uri
                .to_file_path()
                .unwrap()
                .parent()
                .unwrap()
                .join("volpe_parse_tree.txt");
            if let Err(err_msg) = doc.write_tree_to_file(path_buf.as_path()) {
                this.show_error_message(err_msg)
            }
        }
        None => this.show_error_message(format!("{} was not found in documents", uri)),
    };
}

//
// Requests
//

pub type RequestResult<R: request::Request> = Result<R, (lsp_server::ErrorCode, String)>;

pub fn hover_request(this: &mut Server, params: HoverParams) -> RequestResult<Option<Hover>> {
    Ok(this.documents
        .get(
            &params
                .text_document_position_params
                .text_document
                .uri
                .to_string(),
        )
        .map(|doc| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: doc.get_info(),
            }),
            range: None,
        }))
}

pub fn semantic_tokens_full_request(this: &mut Server, params: SemanticTokensParams) -> RequestResult<Option<SemanticTokensResult>> {
    let potential_doc = this.documents.get(&params.text_document.uri.to_string());
    Ok(match potential_doc {
        Some(doc) => {
            let tokens = doc.get_semantic_tokens();
            Some(SemanticTokensResult::Tokens(tokens))
        }
        None => None,
    })
}
