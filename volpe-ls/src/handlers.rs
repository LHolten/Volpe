use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use lsp_server::ErrorCode;
use lsp_types::*;
use volpe_parser::{lexeme_kind::LexemeKind, offset::Offset, syntax::SyntaxIter};

use crate::document::Document;
use crate::lsp_utils::{to_offset, to_position};
use crate::semantic_tokens::{lexeme_to_type, type_index, SemanticTokensBuilder};
use crate::server::Server;
use crate::variable::Variable;

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

pub type RequestResult<R> = Result<R, (ErrorCode, String)>;

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
        .map(|doc| {
            let lexeme = doc
                .parser
                .lexeme_at_offset(to_offset(params.text_document_position_params.position));
            Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::PlainText,
                    value: format!("{:?}", lexeme.kind),
                }),
                range: None,
            }
        });
    Ok(hover)
}

fn get_semantic_tokens(
    doc: &mut Document,
    start: Offset,
    end: Offset,
) -> lsp_types::SemanticTokens {
    fn recurse(
        syntax: SyntaxIter,
        pos: &mut Offset,
        builder: &mut SemanticTokensBuilder,
        vars: &HashMap<Offset, Arc<Variable>>,
        start: Offset,
        end: Offset,
    ) {
        // It's a rule. Recurse for all children.
        for child in syntax {
            // End of range.
            if *pos > end {
                break;
            }
            // Skip ahead.
            if let Some(rule_length) = child.rule_length() {
                if *pos + rule_length < start {
                    builder.skip(rule_length);
                    *pos += rule_length;
                    continue;
                }
            }

            // If there is no rule kind then it is a leaf node - Lexeme.
            if child.kind.is_none() {
                let lexeme = child.lexeme;
                // Convert lexeme to semantic token.
                let maybe_token_type = if matches!(lexeme.kind, LexemeKind::Ident) {
                    vars.get(pos).map(|var| {
                        if var.parameter {
                            SemanticTokenType::PARAMETER
                        } else {
                            SemanticTokenType::VARIABLE
                        }
                    })
                } else {
                    lexeme_to_type(&lexeme.kind)
                };
                // Push it onto the builder.
                if let Some(token_type) = maybe_token_type {
                    builder.push(lexeme.token_length, type_index(token_type), 0);
                    builder.skip(lexeme.length - lexeme.token_length);
                } else {
                    builder.skip(lexeme.length)
                }
                // Adjust position.
                *pos += lexeme.length;
                continue;
            }

            // Otherwise it is a rule and we recurse for all children.
            recurse(child.into_iter(), pos, builder, vars, start, end);
        }
    }

    doc.variable_pass();
    let vars = doc.vars.take().unwrap();
    let mut builder = SemanticTokensBuilder::new();
    let mut pos = Offset::default();
    recurse(
        doc.parser.into_iter(),
        &mut pos,
        &mut builder,
        &vars,
        start,
        end,
    );
    doc.vars = Some(vars);
    builder.build()
}

pub fn semantic_tokens_full_request(
    this: &mut Server,
    params: SemanticTokensParams,
) -> RequestResult<Option<SemanticTokensResult>> {
    let potential_doc = this
        .documents
        .get_mut(&params.text_document.uri.to_string());
    Ok(potential_doc.map(|doc| {
        let tokens = get_semantic_tokens(doc, Offset::default(), Offset::new(u32::MAX, u32::MAX));
        SemanticTokensResult::Tokens(tokens)
    }))
}

pub fn semantic_tokens_range_request(
    this: &mut Server,
    params: SemanticTokensRangeParams,
) -> RequestResult<Option<SemanticTokensRangeResult>> {
    let potential_doc = this
        .documents
        .get_mut(&params.text_document.uri.to_string());
    Ok(potential_doc.map(|doc| {
        let start = to_offset(params.range.start);
        let end = to_offset(params.range.end);
        let tokens = get_semantic_tokens(doc, start, end);
        SemanticTokensRangeResult::Tokens(tokens)
    }))
}

pub fn goto_definition(
    this: &mut Server,
    params: GotoDefinitionParams,
) -> RequestResult<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    Ok(match this.documents.get_mut(&uri.to_string()) {
        Some(doc) => {
            doc.variable_pass();
            let vars = doc.vars.take().unwrap();
            let res = vars
                .get(&to_offset(params.text_document_position_params.position))
                .map(|var| {
                    let pos = to_position(var.declaration);
                    GotoDefinitionResponse::Scalar(Location::new(uri, Range::new(pos, pos)))
                });
            doc.vars = Some(vars);
            res
        }
        None => {
            this.show_error_message(format!("{} was not found in documents", uri));
            None
        }
    })
}
