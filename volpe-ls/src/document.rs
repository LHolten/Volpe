use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::semantic_tokens::{lexeme_to_type, type_index, SemanticTokensBuilder};
use volpe_parser::{offset::Offset, packrat::Parser};

pub struct Document {
    version: i32,
    parser: Parser,
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
    }

    pub fn get_info(&self) -> String {
        format!("version: {}\n{}", self.version, self.parser)
    }

    pub fn write_tree_to_file(&self, path: &Path) -> Result<(), String> {
        let maybe_file = File::create(path);
        if let Err(why) = maybe_file {
            return Err(format!("couldn't open file: {}", why));
        };
        let mut file = maybe_file.unwrap();
        if let Err(why) = file.write_all(self.get_info().as_bytes()) {
            return Err(format!("couldn't write to file: {}", why));
        }
        Ok(())
    }

    pub fn get_semantic_tokens(&self) -> lsp_types::SemanticTokens {
        let mut builder = SemanticTokensBuilder::new();

        if self.parser.0.is_none() {
            return builder.build();
        };

        let mut next_lexemes = vec![self.parser.0.as_ref().unwrap().as_ref()];
        while let Some(lexeme) = next_lexemes.pop() {
            // Follow the tree lexeme by lexeme.
            for rule in &lexeme.rules {
                if rule.length == Offset::default() {
                    continue;
                }
                if let Some(next) = &rule.next {
                    next_lexemes.push(next.as_ref())
                }
            }
            if let Some(next) = &lexeme.next {
                next_lexemes.push(next.as_ref())
            }

            // Convert lexeme to semantic token.
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
        }

        builder.build()
    }
}
