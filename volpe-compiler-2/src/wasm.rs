use volpe_parser_2::ast::{Const, Simple};
use wasm_encoder::{Function, Instruction, ValType};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub expression: Simple,
    pub arg_stack: Vec<Simple>,
}

pub struct CompilerEntry {
    pub signature: Signature,
    pub function: Option<Function>,
}

pub struct Compiler(pub Vec<CompilerEntry>);

pub struct FuncCompiler<'a> {
    pub compiler: &'a mut Compiler,
    pub function: Function,
}

#[derive(Clone)]
pub enum Kind {
    Const(Const),
    Num,
}

impl Kind {
    /// Returns `true` if the kind is [`Kind::Num`].
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
    pub fn compile_new(&mut self, signature: &Signature) -> usize {
        let index = self.0.len();
        self.0.push(CompilerEntry {
            signature: signature.clone(),
            function: None,
        });

        let strict_len = signature.expression.strict_len();
        let mut func_compiler = FuncCompiler {
            compiler: self,
            function: Function::new((0..strict_len).map(|x| (x as u32, ValType::I32))),
        };
        func_compiler.build(signature);
        func_compiler.function.instruction(Instruction::End);

        self.0[index].function = Some(func_compiler.function);
        index
    }
}

impl<'a> FuncCompiler<'a> {
    pub fn build(&mut self, signature: &Signature) -> Kind {
        match &signature.expression {
            Simple::Abs(strict, new_func) => {
                let mut arg_stack = signature.arg_stack.clone();
                let arg = arg_stack.pop().unwrap();
                if *strict {
                    let signature = Signature {
                        expression: new_func.as_ref().clone(),
                        arg_stack,
                    };

                    let f_index = self
                        .compiler
                        .0
                        .iter()
                        .position(|entry| entry.signature == signature)
                        .unwrap_or_else(|| self.compiler.compile_new(&signature));

                    // calculate and push last argument
                    self.build(&Signature {
                        expression: arg,
                        arg_stack: vec![],
                    });
                    // push the rest of the arguments to the stack
                    for i in 0..new_func.strict_len() - 1 {
                        self.function.instruction(Instruction::LocalGet(i as u32));
                    }
                    self.function.instruction(Instruction::Call(f_index as u32)); // need to replace this with tail call
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
                self.function.instruction(Instruction::I32Const(*val));
                self.number(signature)
            }
            Simple::Ident(index) => {
                self.function
                    .instruction(Instruction::LocalGet(*index as u32));
                self.number(signature)
            }
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
            self.function.instruction(Instruction::I32Add);
        }
        Kind::Num
    }
}
