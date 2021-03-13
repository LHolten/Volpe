use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use crate::{lexer::Lexem, offset::Offset};

#[derive(Clone)]
pub enum Syntax {
    Lexem(Rc<Cell<Position>>, bool),
    Rule(Rc<Cell<Rule>>),
}

#[derive(Default, Clone)]
// there is one of these structs for every lexem, and it keeps track of rules
pub struct Position {
    pub lexem: String, // white space and unknown in front
    pub length: Offset,
    pub kind: Lexem,
    pub rules: [Weak<Cell<Rule>>; 9],
    pub next: Weak<Cell<Position>>,
}

// impl Debug for Position {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(&self.lexem)?;
//         if let Some(next) = self.next.upgrade() {
//             next.with(|n| n.fmt(f))
//         } else {
//             Ok(())
//         }
//     }
// }

#[derive(Clone, Default)]
pub struct Rule {
    pub length: Offset,
    pub children: Vec<Syntax>,
    pub success: Option<(Offset, Rc<Cell<Position>>, RuleKind)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RuleKind {
    Expr,
    Stmt,
    App,
    Func,
    Or,
    And,
    Op1,
    Op2,
    Op3,
}
