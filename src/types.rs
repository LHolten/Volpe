use std::collections::HashMap;

use z3::ast::{Ast, Bool, BV};

use crate::core::CoreTerm;

#[derive(Debug, Clone)]
pub struct Func<'ctx> {
    pub arg: CoreTerm,
    pub body: CoreTerm,
    pub scope: HashMap<String, Type<'ctx>>,
}

#[derive(Debug, Clone)]
pub enum Type<'ctx> {
    Bool(Bool<'ctx>),
    BV(BV<'ctx>),
    Func(Vec<(Bool<'ctx>, Func<'ctx>)>),
}

impl<'ctx> Type<'ctx> {
    pub fn as_bv(self) -> Option<BV<'ctx>> {
        if let Type::BV(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_bool(self) -> Option<Bool<'ctx>> {
        if let Type::Bool(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_func(self) -> Option<Vec<(Bool<'ctx>, Func<'ctx>)>> {
        if let Type::Func(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn ite(cond: &Bool<'ctx>, a: &Type<'ctx>, b: &Type<'ctx>) -> Option<Self> {
        match (a, b) {
            (Type::Bool(a), Type::Bool(b)) => Some(cond.ite(a, b).into()),
            (Type::BV(a), Type::BV(b)) => Some(cond.ite(a, b).into()),
            (Type::Func(a), Type::Func(b)) => Some(Self::Func(
                a.iter()
                    .cloned()
                    .map(|(c, f)| (Bool::and(c.get_ctx(), &[&c, cond]), f))
                    .chain(b.iter().cloned())
                    .collect(),
            )),
            _ => None,
        }
    }
}

impl<'ctx> From<Bool<'ctx>> for Type<'ctx> {
    fn from(val: Bool<'ctx>) -> Self {
        Self::Bool(val)
    }
}

impl<'ctx> From<BV<'ctx>> for Type<'ctx> {
    fn from(val: BV<'ctx>) -> Self {
        Self::BV(val)
    }
}
