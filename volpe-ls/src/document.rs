use crate::semantic_tokens::{lexeme_to_type, type_index, SemanticTokensBuilder};
use lsp_types;
use volpe_parser::{offset::Offset, syntax::Syntax, with_internal::WithInternal};

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

        if let Some(mut syntax) = self.syntax.take() {
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

    pub fn get_semantic_tokens(&mut self) -> lsp_types::SemanticTokens {
        let mut builder = SemanticTokensBuilder::new();
        if let Some(syntax) = self.syntax.take() {
            let mut potential_lexeme = Some(syntax.get_pos());
            while let Some(lexeme) = potential_lexeme {
                let length = lexeme.with(|l| l.length);
                if let Some(token_type) = lexeme.with(lexeme_to_type) {
                    let whitespace = lexeme.with(|l| {
                        let mut offset = Offset::new(0, 0);
                        for c in l.lexem.chars() {
                            if !c.is_ascii_whitespace() {
                                break;
                            }
                            if c == '\n' {
                                offset.line += 1;
                                offset.char = 0;
                            } else {
                                offset.char += 1;
                            }
                        }
                        offset
                    });
                    builder.skip(whitespace);
                    builder.push(length - whitespace, type_index(token_type), 0);
                } else {
                    builder.skip(length)
                }
                potential_lexeme = lexeme.with(|l| l.next.upgrade());
            }
            self.syntax = Some(syntax);
        }
        builder.build()
    }
}
