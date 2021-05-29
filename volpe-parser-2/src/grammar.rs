use crate::lexeme_kind::LexemeKind;

impl LexemeKind {
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
            || self.reduce_brackets()
    }

    pub fn reduce_bit_or(&self) -> bool {
        matches!(self, LexemeKind::BitOr) || self.reduce_bit_and()
    }

    pub fn reduce_bit_and(&self) -> bool {
        matches!(self, LexemeKind::BitAnd) || self.reduce_brackets()
    }

    pub fn reduce_bit_xor(&self) -> bool {
        matches!(self, LexemeKind::BitXor) || self.reduce_brackets()
    }

    pub fn reduce_bit_shift(&self) -> bool {
        matches!(self, LexemeKind::BitShl | LexemeKind::BitShr) || self.reduce_brackets()
    }

    pub fn reduce_brackets(&self) -> bool {
        matches!(
            self,
            LexemeKind::LBrace
                | LexemeKind::LCurlyBrace
                | LexemeKind::Ident
                | LexemeKind::Num
                | LexemeKind::Error
        )
    }
}

impl LexemeKind {
    pub fn child_count(&self) -> usize {
        match self {
            LexemeKind::LBrace => 1,
            LexemeKind::RBrace => 1,
            LexemeKind::LCurlyBrace => 1,
            LexemeKind::RCurlyBrace => 1,
            LexemeKind::Ident => 0,
            LexemeKind::Num => 0,
            LexemeKind::Error => 0,
            _ => 2,
        }
    }

    pub fn reduce(&self, new: &Self) -> bool {
        match self {
            LexemeKind::App => new.reduce_or(),
            LexemeKind::Plus => new.reduce_multiplicative(),
            LexemeKind::Minus => new.reduce_multiplicative(),
            LexemeKind::Num => false,
            LexemeKind::Ident => false,
            LexemeKind::Error => false,
            _ => unimplemented!(),
        }
    }
}
