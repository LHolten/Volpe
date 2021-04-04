use std::fs::File;
use std::io::Write;

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

pub fn did_open_text_document_notification(this: &mut Server, params: DidOpenTextDocumentParams) {
    let doc = Document::new(&params);
    if let Err(err_msg) = write_tree_to_file(&doc, &params.text_document.uri) {
        this.show_error_message(err_msg)
    };
    this.documents
        .insert(params.text_document.uri.to_string(), doc);
}

pub fn did_change_text_document_notification(
    this: &mut Server,
    params: DidChangeTextDocumentParams,
) {
    let uri = params.text_document.uri.to_string();
    match this.documents.get_mut(&uri) {
        Some(doc) => {
            doc.update(&params);
            if let Err(err_msg) = write_tree_to_file(doc, &params.text_document.uri) {
                this.show_error_message(err_msg)
            }
        }
        None => this.show_error_message(format!("{} was not found in documents", uri)),
    };
}

//
// Requests
//

pub type RequestResult<R> = Result<R, (lsp_server::ErrorCode, String)>;

pub fn hover_request(this: &mut Server, params: HoverParams) -> RequestResult<Option<Hover>> {
    let hover = this
        .documents
        .get(
            &params
                .text_document_position_params
                .text_document
                .uri
                .to_string(),
        )
        .map(|_doc| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: "TODO".to_string(),
            }),
            range: None,
        });
    Ok(hover)
}

use crate::semantic_tokens::{lexeme_to_type, type_index, SemanticTokensBuilder};
use volpe_parser::{lexeme_kind::LexemeKind, offset::Offset};

fn get_semantic_tokens(doc: &Document) -> lsp_types::SemanticTokens {
    let mut builder = SemanticTokensBuilder::new();

    let mut pos = Offset::default();
    let mut next_lexemes = vec![doc.parser.0.as_ref()];
    while let Some(lexeme) = next_lexemes.pop() {
        // Follow the tree lexeme by lexeme.
        for rule in &lexeme.rules {
            if rule.length == Offset::default() {
                continue;
            }
            if let Some(next) = &rule.next {
                next_lexemes.push(next);
            }
        }
        if let Some(next) = &lexeme.next {
            next_lexemes.push(next);
        }

        // Convert lexeme to semantic token.
        let maybe_token_type = if matches!(lexeme.kind, LexemeKind::Ident) {
            let name = &lexeme.string[..lexeme.token_length.char as usize];
            let mut token_type = None;
            // TODO Make more efficient with better data structure
            for var in doc.vars.iter() {
                if var.name == name && (var.declaration == pos || var.locations.contains(&pos)) {
                    token_type = Some(if var.parameter {
                        SemanticTokenType::PARAMETER
                    } else {
                        SemanticTokenType::VARIABLE
                    });
                    break;
                }
            }
            token_type
        } else {
            lexeme_to_type(&lexeme.kind)
        };

        if let Some(token_type) = maybe_token_type {
            builder.push(lexeme.token_length, type_index(token_type), 0);
            builder.skip(lexeme.length - lexeme.token_length);
        } else {
            builder.skip(lexeme.length)
        }

        pos += lexeme.length;
    }

    builder.build()
}

pub fn semantic_tokens_full_request(
    this: &mut Server,
    params: SemanticTokensParams,
) -> RequestResult<Option<SemanticTokensResult>> {
    let potential_doc = this.documents.get(&params.text_document.uri.to_string());
    Ok(match potential_doc {
        Some(doc) => {
            let tokens = get_semantic_tokens(&doc);
            Some(SemanticTokensResult::Tokens(tokens))
        }
        None => None,
    })
}
