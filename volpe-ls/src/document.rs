use std::borrow::Borrow;

use lsp_types;
use volpe_parser::{offset::Offset, packrat::Syntax};

pub struct Document {
    version: i32,
    syntax: Option<Syntax>,
}

impl Document {
    pub fn new(params: &lsp_types::DidOpenTextDocumentParams) -> Document {
        let syntax = Syntax::default().parse(
            &params.text_document.text,
            Offset::new(0, 0),
            Offset::new(0, 0),
        );
        Document {
            version: params.text_document.version,
            syntax: Some(syntax),
        }
    }

    pub fn update(&mut self, params: &lsp_types::DidChangeTextDocumentParams) {
        self.version = params.text_document.version;

        let maybe_syntax = self.syntax.take();
        if let Some(mut syntax) = maybe_syntax {
            for event in params.content_changes.iter() {
                if let Some(range) = event.range {
                    // TODO fix utf-8 and utf-16 mismatch
                    let start =
                        Offset::new(range.start.line as usize, range.start.character as usize);
                    let end = Offset::new(range.end.line as usize, range.end.character as usize);
                    syntax = syntax.parse(&event.text, start, end - start);
                }
            }
            self.syntax = Some(syntax);
        }
    }

    pub fn get_info(&self) -> String {
        match &self.syntax {
            Some(s) => format!("version: {}\nsyntax: {:#?}", self.version, s),
            None => format!("version: {}", self.version),
        }
    }
}
