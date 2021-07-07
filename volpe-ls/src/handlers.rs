use std::fs::File;
use std::io::Write;

use lsp_server::ErrorCode;
use lsp_types::*;

use crate::document::Document;
use crate::server::Server;

//
// Notifications
//

fn write_tree_to_file(doc: &Document, document_uri: &Url) -> Result<(), String> {
    let path = document_uri
        .to_file_path()
        .map_err(|_| "can't convert file uri to file path")?
        .parent()
        .ok_or("couldn't get parent")?
        .join("volpe_parse_tree.txt");
    let mut file = File::create(path).map_err(|why| format!("couldn't open file: {}", why))?;
    file.write_all(doc.get_info().as_bytes())
        .map_err(|why| format!("couldn't write to file: {}", why))
}

fn with_doc<T, F: FnOnce(&mut Document) -> T>(
    this: &mut Server,
    uri: String,
    func: F,
) -> Option<T> {
    let potential_doc = this.documents.get_mut(&uri);
    if let Some(doc) = potential_doc {
        Some(func(doc))
    } else {
        this.show_error_message(format!("{} was not found in documents", uri));
        None
    }
}

fn diagnostics(this: &mut Server, uri: Url) {
    if let Some((diagnostics, version)) = with_doc(this, uri.to_string(), |doc| {
        (doc.get_diagnostics(), doc.version)
    }) {
        this.send_notification::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: Some(version),
        })
    }
}

pub fn did_open_text_document_notification(this: &mut Server, params: DidOpenTextDocumentParams) {
    let doc = Document::new(&params);
    if let Err(err_msg) = write_tree_to_file(&doc, &params.text_document.uri) {
        this.show_error_message(err_msg)
    };
    this.documents
        .insert(params.text_document.uri.to_string(), doc);
    diagnostics(this, params.text_document.uri);
}

pub fn did_change_text_document_notification(
    this: &mut Server,
    params: DidChangeTextDocumentParams,
) {
    if let Some(Err(err_msg)) = with_doc(this, params.text_document.uri.to_string(), |doc| {
        doc.update(&params);
        write_tree_to_file(doc, &params.text_document.uri)
    }) {
        this.show_error_message(err_msg);
    }
    diagnostics(this, params.text_document.uri);
}

pub fn did_save_text_document_notification(this: &mut Server, params: DidSaveTextDocumentParams) {
    diagnostics(this, params.text_document.uri);
}

//
// Requests
//

pub type RequestResult<R> = Result<R, (ErrorCode, String)>;

pub fn hover(_this: &mut Server, _params: HoverParams) -> RequestResult<Option<Hover>> {
    // TODO
    Ok(None)
}

pub fn semantic_tokens_full(
    _this: &mut Server,
    _params: SemanticTokensParams,
) -> RequestResult<Option<SemanticTokensResult>> {
    // TODO
    Ok(None)
}

pub fn semantic_tokens_range(
    _this: &mut Server,
    _params: SemanticTokensRangeParams,
) -> RequestResult<Option<SemanticTokensRangeResult>> {
    // TODO
    Ok(None)
}

pub fn goto_definition(
    _this: &mut Server,
    _params: GotoDefinitionParams,
) -> RequestResult<Option<GotoDefinitionResponse>> {
    // TODO
    Ok(None)
}
