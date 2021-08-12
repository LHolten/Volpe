use std::mem::take;

use volpe_parser_2::ast::{Const, Simple};
use wasm_encoder::{Function, Instruction};

pub struct Compiler {
    pub stack: Vec<Simple>,
    pub func: Function,
}

#[derive(Clone)]
pub enum Kind {
    Const(Const),
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
    pub fn build_no_stack(&mut self, expr: &Simple) -> Kind {
        let backup = take(&mut self.stack);
        let res = self.build(expr);
        self.stack = backup;
        res
    }

    pub fn build(&mut self, expr: &Simple) -> Kind {
        match expr {
            Simple::Abs(strict, func) => {
                if *strict {
                    todo!()
                } else {
                    let val = self.stack.pop().unwrap();
                    self.build(&func.replace(0, &val))
                }
            }
            Simple::App([func, arg]) => {
                self.stack.push(arg.as_ref().clone());
                self.build(func)
            }
            Simple::Const(val) => Kind::Const(val.clone()),
            Simple::Num(val) => {
                self.func.instruction(Instruction::I32Const(*val));
                while let Some(op) = self.stack.pop() {
                    assert!(self.build_no_stack(&op).is_const());
                    let second = self.stack.pop().unwrap();
                    assert!(self.build_no_stack(&second).is_num());
                    self.func.instruction(Instruction::I32Add);
                }
                Kind::Num
            }
            _ => unreachable!(),
        }
    }
}
