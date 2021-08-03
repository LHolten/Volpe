use std::{mem::take, rc::Rc};

use string_interner::DefaultSymbol;
use volpe_parser_2::ast::{Simple, TerminalKind};
use wasm_encoder::{Function, Instruction};

#[derive(Clone)]
pub struct Closure {
    pub val: Rc<Simple>,
    pub env: Vec<EnvItem>,
}

#[derive(Clone)]
pub struct EnvItem {
    ident: DefaultSymbol,
    closure: Closure,
}

pub struct Compiler {
    pub stack: Vec<Closure>,
    pub func: Function,
}

pub enum Kind {
    Const(DefaultSymbol),
    Num,
}

impl Kind {
    /// Returns `true` if the kind is [`Num`].
    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num)
    }

    /// Returns `true` if the kind is [`Const`].
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(..))
    }
}

impl Compiler {
    pub fn build_no_stack(&mut self, expr: Closure) -> Kind {
        let backup = take(&mut self.stack);
        let res = self.build(expr);
        self.stack = backup;
        res
    }

    pub fn build(&mut self, mut expr: Closure) -> Kind {
        match expr.val.as_ref() {
            Simple::Func([arg, func]) => {
                expr.env.push(EnvItem {
                    ident: arg.as_ident(),
                    closure: self.stack.pop().unwrap(),
                });
                expr.val = func.clone();
                self.build(expr)
            }
            Simple::App([func, arg]) => {
                self.stack.push(Closure {
                    val: arg.clone(),
                    env: expr.env.clone(),
                });
                expr.val = func.clone();
                self.build(expr)
            }
            Simple::Terminal { kind, val } => match kind {
                TerminalKind::Num => {
                    self.func.instruction(Instruction::I32Const(1));
                    if let Some(op) = self.stack.pop() {
                        assert!(self.build_no_stack(op).is_const());
                        let second = self.stack.pop().unwrap();
                        assert!(self.build_no_stack(second).is_num());
                        self.func.instruction(Instruction::I32Add);
                    }
                    Kind::Num
                }
                TerminalKind::Ident => self.build(
                    expr.env
                        .into_iter()
                        .find(|i| &i.ident == val)
                        .unwrap()
                        .closure,
                ),
                TerminalKind::Const => {
                    assert!(self.stack.is_empty());
                    Kind::Const(*val)
                }
            },
        }
    }
}
