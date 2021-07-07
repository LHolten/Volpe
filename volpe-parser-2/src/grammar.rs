use crate::lexeme_kind::LexemeKind;

impl LexemeKind {
    pub fn reduce_object(&self) -> bool {
        matches!(self, LexemeKind::Comma) || self.reduce_item()
    }

    pub fn reduce_item(&self) -> bool {
        matches!(self, LexemeKind::Colon) || self.reduce_app()
    }

    pub fn reduce_expr(&self) -> bool {
        matches!(self, LexemeKind::Semicolon) || self.reduce_stmt()
    }

    pub fn reduce_stmt(&self) -> bool {
        matches!(self, LexemeKind::Assign | LexemeKind::Ite) || self.reduce_app()
    }

    pub fn reduce_app(&self) -> bool {
        matches!(self, LexemeKind::App) || self.reduce_func()
    }

    pub fn reduce_func(&self) -> bool {
        matches!(self, LexemeKind::Func) || self.reduce_or()
    }

    pub fn reduce_or(&self) -> bool {
        matches!(self, LexemeKind::Or) || self.reduce_and()
    }

    pub fn reduce_and(&self) -> bool {
        matches!(self, LexemeKind::And) || self.reduce_equality() || self.reduce_comparative()
    }

    pub fn reduce_equality(&self) -> bool {
        matches!(self, LexemeKind::Equals | LexemeKind::UnEquals)
            || self.reduce_additive()
            || self.reduce_bit_xor()
            || self.reduce_bit_or()
            || self.reduce_bit_shift()
    }

    pub fn reduce_comparative(&self) -> bool {
        matches!(
            self,
            LexemeKind::Less
                | LexemeKind::Greater
                | LexemeKind::LessEqual
                | LexemeKind::GreaterEqual
        ) || self.reduce_additive()
    }

    pub fn reduce_additive(&self) -> bool {
        matches!(self, LexemeKind::Plus | LexemeKind::Minus) || self.reduce_multiplicative()
    }

    pub fn reduce_multiplicative(&self) -> bool {
        matches!(self, LexemeKind::Mul | LexemeKind::Div | LexemeKind::Mod)
    }

    pub fn reduce_bit_or(&self) -> bool {
        matches!(self, LexemeKind::BitOr) || self.reduce_bit_and()
    }

    pub fn reduce_bit_and(&self) -> bool {
        matches!(self, LexemeKind::BitAnd)
    }

    pub fn reduce_bit_xor(&self) -> bool {
        matches!(self, LexemeKind::BitXor)
    }

    pub fn reduce_bit_shift(&self) -> bool {
        matches!(self, LexemeKind::BitShl | LexemeKind::BitShr)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RuleKind {
    OpeningBracket,
    ClosingBracket,
    Terminal,
    Operator,
}

impl LexemeKind {
    pub fn rule_kind(&self) -> RuleKind {
        match self {
            LexemeKind::LRoundBracket => RuleKind::OpeningBracket,
            LexemeKind::RRoundBracket => RuleKind::ClosingBracket,
            LexemeKind::LCurlyBracket => RuleKind::OpeningBracket,
            LexemeKind::RCurlyBracket => RuleKind::ClosingBracket,
            LexemeKind::Ident => RuleKind::Terminal,
            LexemeKind::Num => RuleKind::Terminal,
            LexemeKind::Error => RuleKind::ClosingBracket,
            _ => RuleKind::Operator,
        }
    }

    pub fn reduce(&self, new: &Self) -> bool {
        match self {
            LexemeKind::App => new.reduce_or(),
            LexemeKind::Plus => new.reduce_multiplicative(),
            LexemeKind::Minus => new.reduce_multiplicative(),
            LexemeKind::Num => unreachable!(),
            LexemeKind::Ident => unreachable!(),
            LexemeKind::Error => true,
            LexemeKind::LRoundBracket => true,
            LexemeKind::LCurlyBracket => true,
            LexemeKind::Semicolon => new.reduce_expr(),
            LexemeKind::Assign => new.reduce_stmt(),
            LexemeKind::Ite => new.reduce_stmt(),
            LexemeKind::Func => new.reduce_func(),
            LexemeKind::Or => new.reduce_and(),
            LexemeKind::And => new.reduce_equality() || new.reduce_comparative(),
            LexemeKind::Equals => new.reduce_equality(),
            LexemeKind::UnEquals => new.reduce_equality(),
            LexemeKind::Less => new.reduce_comparative(),
            LexemeKind::Greater => new.reduce_comparative(),
            LexemeKind::LessEqual => new.reduce_comparative(),
            LexemeKind::GreaterEqual => new.reduce_comparative(),
            LexemeKind::BitOr => new.reduce_bit_and(),
            LexemeKind::BitAnd => false,
            LexemeKind::BitXor => false,
            LexemeKind::BitShl => false,
            LexemeKind::BitShr => false,
            LexemeKind::Mul => false,
            LexemeKind::Div => false,
            LexemeKind::Mod => false,
            LexemeKind::RRoundBracket => unreachable!(),
            LexemeKind::RCurlyBracket => unreachable!(),
            LexemeKind::Colon => new.reduce_item(),
            LexemeKind::Comma => new.reduce_object(),
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
