// Basically same as rust-analyzer, except simpler
// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/rust-analyzer/src/semantic_tokens.rs

#![allow(dead_code)]

use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens};
use volpe_parser_2::{lexeme_kind::LexemeKind, offset::Offset};

pub const SUPPORTED_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::OPERATOR,
];

pub const SUPPORTED_MODIFIERS: &[SemanticTokenModifier] = &[];

pub fn type_index(token_type: SemanticTokenType) -> u32 {
    SUPPORTED_TYPES
        .iter()
        .position(|it| *it == token_type)
        .unwrap() as u32
}

pub struct SemanticTokensBuilder {
    prev: Offset,
    current: Offset,
    data: Vec<SemanticToken>,
}

impl SemanticTokensBuilder {
    pub fn new() -> Self {
        SemanticTokensBuilder {
            prev: Offset::new(0, 0),
            current: Offset::new(0, 0),
            data: Vec::new(),
        }
    }

    pub fn skip(&mut self, length: Offset) {
        self.current += length;
    }

    pub fn push(&mut self, length: Offset, token_index: u32, modifier_bitset: u32) {
        let diff = self.current - self.prev;
        self.prev = self.current;
        self.current += length;

        let token = SemanticToken {
            delta_line: diff.line as u32,
            delta_start: diff.char as u32,
            length: length.char as u32,
            token_type: token_index,
            token_modifiers_bitset: modifier_bitset,
        };

        self.data.push(token);
    }

    pub fn build(self) -> SemanticTokens {
        SemanticTokens {
            result_id: None,
            data: self.data,
        }
    }
}

pub fn lexeme_to_type(kind: &LexemeKind) -> Option<SemanticTokenType> {
    match kind {
        LexemeKind::Ident => Some(SemanticTokenType::VARIABLE),
        LexemeKind::Num => Some(SemanticTokenType::NUMBER),
        LexemeKind::Operator => Some(SemanticTokenType::OPERATOR),
        LexemeKind::Raw => Some(SemanticTokenType::STRING),
        _ => None,
    }
}
