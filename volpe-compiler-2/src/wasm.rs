use std::{collections::HashMap, mem::swap};

use volpe_parser_2::ast::{Const, Simple};
use wasm_encoder::{Function, Instruction, ValType};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub expression: Simple,
    pub arg_stack: Vec<Simple>,
}

pub struct Compiler {
    pub signatures: HashMap<Signature, usize>,
    pub functions: Vec<(Signature, Function)>,
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

    pub fn as_const(&self) -> Const {
        match self {
            Kind::Const(c) => c.clone(),
            Kind::Num => unreachable!(),
        }
    }
}

impl Compiler {
    pub fn with_func(&mut self, func: &mut Function, f: impl Fn(&mut Self) -> Kind) -> Kind {
        swap(&mut self.func, func);
        let res = f(self);
        swap(&mut self.func, func);
        res
    }

    pub fn compile_new(&mut self, signature: &Signature) -> usize {
        let index = self.functions.len();
        self.signatures.insert(signature.clone(), index);
        self.functions
            .push((signature.clone(), Function::new(vec![])));

        let strict_len = signature.expression.strict_len();
        let mut new_func = Function::new((0..strict_len).map(|x| (x, ValType::I32)));
        self.with_func(&mut new_func, |compiler| compiler.build(signature));
        new_func.instruction(Instruction::End);
        self.functions[index].1 = new_func;

        index
    }

    pub fn build(&mut self, signature: &Signature) -> Kind {
        match &signature.expression {
            Simple::Abs(strict, new_func) => {
                let mut arg_stack = signature.arg_stack.clone();
                let arg = arg_stack.pop().unwrap();
                if *strict {
                    let strict_len = new_func.strict_len();
                    let signature = Signature {
                        expression: new_func.replace(0, &Simple::Strict(strict_len)),
                        arg_stack,
                    };

                    let f_index = self
                        .signatures
                        .get(&signature)
                        .copied()
                        .unwrap_or_else(|| self.compile_new(&signature));

                    // push all arguments to the stack
                    for i in 0..strict_len {
                        self.func.instruction(Instruction::LocalGet(i));
                    }
                    // calculate and push last argument
                    self.build(&Signature {
                        expression: arg,
                        arg_stack: vec![],
                    });
                    self.func.instruction(Instruction::Call(f_index as u32)); // need to replace this with tail call
                    Kind::Num
                } else {
                    self.build(&Signature {
                        expression: new_func.replace(0, &arg),
                        arg_stack,
                    })
                }
            }
            Simple::App([new_func, arg]) => {
                let mut arg_stack = signature.arg_stack.clone();
                arg_stack.push(arg.as_ref().clone());
                self.build(&Signature {
                    expression: new_func.as_ref().clone(),
                    arg_stack,
                })
            }
            Simple::Const(val) => {
                assert!(signature.arg_stack.is_empty());
                Kind::Const(val.clone())
            }
            Simple::Num(val) => {
                self.func.instruction(Instruction::I32Const(*val));
                self.number(signature)
            }
            Simple::Strict(index) => {
                self.func.instruction(Instruction::LocalGet(*index));
                self.number(signature)
            }
            Simple::Ident(_) => unreachable!(),
        }
    }

    pub fn number(&mut self, signature: &Signature) -> Kind {
        let mut arg_stack = signature.arg_stack.clone();
        while let Some(op) = arg_stack.pop() {
            let _op = self
                .build(&Signature {
                    expression: op,
                    arg_stack: vec![],
                })
                .as_const();
            assert!(self
                .build(&Signature {
                    expression: arg_stack.pop().unwrap(),
                    arg_stack: vec![],
                })
                .is_num());
            self.func.instruction(Instruction::I32Add);
        }
        Kind::Num
    }
}
