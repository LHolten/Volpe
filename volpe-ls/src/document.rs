use std::collections::HashMap;
use std::sync::Arc;

use crate::variable::Variable;
use volpe_parser::{offset::Offset, packrat::Parser};

pub struct Document {
    pub version: i32,
    pub parser: Parser,
    pub vars: Option<HashMap<Offset, Arc<Variable>>>,
}

impl Document {
    pub fn new(params: &lsp_types::DidOpenTextDocumentParams) -> Document {
        let mut parser = Parser::default();
        parser.parse(
            &params.text_document.text,
            Offset::default(),
            Offset::default(),
        );
        Document {
            version: params.text_document.version,
            vars: None,
            parser,
        }
    }

    pub fn update(&mut self, params: &lsp_types::DidChangeTextDocumentParams) {
        self.version = params.text_document.version;

        for event in params.content_changes.iter() {
            if let Some(range) = event.range {
                // TODO fix utf-8 and utf-16 mismatch
                let start = Offset::new(range.start.line, range.start.character);
                let end = Offset::new(range.end.line, range.end.character);
                self.parser.parse(&event.text, start, end - start);
            }
        }

        self.vars = None;
    }

    pub fn get_info(&self) -> String {
        format!("version: {}\n{}", self.version, self.parser)
    }
}
