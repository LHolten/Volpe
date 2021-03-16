use crate::semantic_tokens::{lexeme_to_type, type_index, SemanticTokensBuilder};
use lsp_types;
use volpe_parser::{internal::UpgradeInternal, offset::Offset, packrat::{Packrat, first_lexeme}};

pub struct Document {
    version: i32,
    packrat: Packrat,
}

impl Document {
    pub fn new(params: &lsp_types::DidOpenTextDocumentParams) -> Document {
        let mut packrat = Packrat::default();
        packrat.parse(
            &params.text_document.text,
            Offset::new(0, 0),
            Offset::new(0, 0),
        );
        Document {
            version: params.text_document.version,
            packrat,
        }
    }

    pub fn update(&mut self, params: &lsp_types::DidChangeTextDocumentParams) {
        self.version = params.text_document.version;

        for event in params.content_changes.iter() {
            if let Some(range) = event.range {
                // TODO fix utf-8 and utf-16 mismatch
                let start = Offset::new(range.start.line as usize, range.start.character as usize);
                let end = Offset::new(range.end.line as usize, range.end.character as usize);
                self.packrat.parse(&event.text, start, end - start);
            }
        }
    }

    pub fn get_info(&self) -> String {
        format!("version: {}\nsyntax: {:#?}", self.version, self.packrat)
    }

    pub fn get_semantic_tokens(&self) -> lsp_types::SemanticTokens {
        let mut builder = SemanticTokensBuilder::new();
        let mut potential_lexeme = Some(first_lexeme(&self.packrat.0));
        while let Some(lexeme) = potential_lexeme {
            if let Some(token_type) = lexeme_to_type(&lexeme.kind) {
                let mut whitespace = Offset::new(0, 0);
                for c in lexeme.string.chars() {
                    // TODO This does not correctly account for comments
                    if !c.is_ascii_whitespace() {
                        break;
                    }
                    if c == '\n' {
                        whitespace.line += 1;
                        whitespace.char = 0;
                    } else {
                        whitespace.char += 1;
                    }
                }
                builder.skip(whitespace);
                builder.push(lexeme.length - whitespace, type_index(token_type), 0);
            } else {
                builder.skip(lexeme.length)
            }
            potential_lexeme = lexeme.next.upgrade();
        }
        builder.build()
    }
}
