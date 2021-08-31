use std::cmp::Ordering;

use crate::lexeme_kind::LexemeKind;

#[derive(Debug, Clone, Copy)]
pub enum RuleKind {
    OpeningBracket,
    ClosingBracket,
    Terminal,
    Operator,
}

impl LexemeKind {
    pub fn priority(&self) -> usize {
        match self {
            LexemeKind::Semicolon => 0,
            LexemeKind::Assign => 0,
            LexemeKind::App => 2,
            LexemeKind::Abs => 9,
            LexemeKind::Case => 9,
            _ => unreachable!(),
        }
    }

    pub fn rule_kind(&self) -> RuleKind {
        match self {
            LexemeKind::LRoundBracket => RuleKind::OpeningBracket,
            LexemeKind::RRoundBracket => RuleKind::ClosingBracket,
            LexemeKind::LCurlyBracket => RuleKind::OpeningBracket,
            LexemeKind::RCurlyBracket => RuleKind::ClosingBracket,
            LexemeKind::Ident => RuleKind::Terminal,
            LexemeKind::Unique => RuleKind::Terminal,
            LexemeKind::Num => RuleKind::Terminal,
            LexemeKind::Wasm => RuleKind::Terminal,
            LexemeKind::Error => RuleKind::ClosingBracket,
            _ => RuleKind::Operator,
        }
    }

    pub fn stack_can_hold(&self, new: &Self) -> bool {
        match self.rule_kind() {
            RuleKind::OpeningBracket => true,
            RuleKind::Operator => match new.rule_kind() {
                RuleKind::ClosingBracket => false,
                RuleKind::Operator => match self.priority().cmp(&new.priority()) {
                    Ordering::Less => true,
                    Ordering::Equal => matches!(self.priority(), 0 | 9),
                    Ordering::Greater => false,
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn bracket_counterpart(&self) -> LexemeKind {
        assert!(matches!(
            self.rule_kind(),
            RuleKind::OpeningBracket | RuleKind::ClosingBracket
        ));
        match self {
            LexemeKind::LCurlyBracket => LexemeKind::RCurlyBracket,
            LexemeKind::LRoundBracket => LexemeKind::RRoundBracket,
            LexemeKind::RCurlyBracket => LexemeKind::LCurlyBracket,
            LexemeKind::RRoundBracket => LexemeKind::LRoundBracket,
            _ => unreachable!(),
        }
    }
}
