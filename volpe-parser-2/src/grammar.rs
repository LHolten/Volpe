use crate::lexeme_kind::LexemeKind;

#[derive(Debug, Clone, Copy)]
pub enum RuleKind {
    OpeningBracket,
    ClosingBracket,
    Terminal,
    Semicolon,
}

impl LexemeKind {
    pub fn rule_kind(&self) -> RuleKind {
        match self {
            LexemeKind::LRoundBracket => RuleKind::OpeningBracket,
            LexemeKind::RRoundBracket => RuleKind::ClosingBracket,
            LexemeKind::LCurlyBracket => RuleKind::OpeningBracket,
            LexemeKind::RCurlyBracket => RuleKind::ClosingBracket,
            LexemeKind::Semicolon => RuleKind::Semicolon,
            LexemeKind::Ident => RuleKind::Terminal,
            LexemeKind::Operator => RuleKind::Terminal,
            LexemeKind::Num => RuleKind::Terminal,
            LexemeKind::Wasm => RuleKind::Terminal,
            LexemeKind::Error => unreachable!(),
            k => todo!("lexeme kind has not been implemented {:?}", k),
        }
    }

    pub fn bracket_counterpart(&self) -> LexemeKind {
        match self {
            LexemeKind::LCurlyBracket => LexemeKind::RCurlyBracket,
            LexemeKind::LRoundBracket => LexemeKind::RRoundBracket,
            LexemeKind::RCurlyBracket => LexemeKind::LCurlyBracket,
            LexemeKind::RRoundBracket => LexemeKind::LRoundBracket,
            _ => unreachable!(),
        }
    }
}
