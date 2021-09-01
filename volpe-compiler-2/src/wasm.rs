use string_interner::DefaultSymbol;
use volpe_parser_2::ast::Simple;
use wasm_encoder::{Function, Instruction, ValType};
use wast::{
    parser::{self, ParseBuffer},
    Encode, Expression,
};

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
    Const(DefaultSymbol),
    Num,
}

impl Kind {
    /// Returns `true` if the kind is [`Kind::Num`].
    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num)
    }

    pub fn as_const(&self) -> DefaultSymbol {
        match self {
            Kind::Const(c) => *c,
            _ => unreachable!(),
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

        let strict_len = signature.expression.strict_len() + signature.arg_stack.len();
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
            Simple::Abs(new_func) => {
                let mut arg_stack = signature.arg_stack.clone();
                let arg = arg_stack.pop().unwrap();

                self.build(&Signature {
                    expression: new_func.replace(0, &arg),
                    arg_stack,
                })
            }
            Simple::App([new_func, arg]) => {
                let mut arg_stack = signature.arg_stack.clone();
                arg_stack.push(arg.as_ref().clone());
                self.build(&Signature {
                    expression: new_func.as_ref().clone(),
                    arg_stack,
                })
            }
            Simple::Unique(val) => {
                assert!(signature.arg_stack.is_empty());
                Kind::Const(*val)
            }
            Simple::Num(val) => {
                assert!(signature.arg_stack.is_empty());
                self.function.instruction(Instruction::I32Const(*val));
                Kind::Num
            }
            Simple::Strict(index) => {
                // this should only happen inside strict functions
                self.function
                    .instruction(Instruction::LocalGet(*index as u32));
                Kind::Num
            }
            Simple::Ident(_) => unreachable!(),
            Simple::Case([c, body]) => {
                let mut arg_stack = signature.arg_stack.clone();
                let alternative = arg_stack.pop().unwrap();
                let value = arg_stack.pop().unwrap();
                let c = self.build(&Signature {
                    expression: c.as_ref().clone(),
                    arg_stack: vec![],
                });
                let value_res = self.build(&Signature {
                    expression: value.clone(),
                    arg_stack: vec![],
                });
                if value_res.as_const() == c.as_const() {
                    self.build(&Signature {
                        expression: body.as_ref().clone(),
                        arg_stack,
                    })
                } else {
                    arg_stack.push(value); // bring back the value for the next case
                    self.build(&Signature {
                        expression: alternative,
                        arg_stack,
                    })
                }
            }
            Simple::Wasm(wat) => {
                let wasm_count = wat[1..2].parse::<usize>().unwrap();
                let wat = ParseBuffer::new(&wat[3..wat.len() - 1]).unwrap();
                let expr = parser::parse::<Expression>(&wat).unwrap();

                let branch_count = expr
                    .instrs
                    .iter()
                    .filter_map(|instr| {
                        if let wast::Instruction::ReturnCall(index) = instr {
                            if let wast::Index::Num(num, _) = index.0.unwrap_index() {
                                Some(*num + 1)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(0);

                let mut args = signature.arg_stack.clone();
                let branches = args.split_off(args.len() - branch_count as usize);
                let wasm_args = args.split_off(args.len() - wasm_count);

                // push wasm args to the stack for usage by inline wasm
                // these are inner first
                for arg in wasm_args.into_iter().rev() {
                    assert!(self
                        .build(&Signature {
                            expression: arg,
                            arg_stack: vec![],
                        })
                        .is_num());
                }

                expr.instrs.iter().for_each(|instr| {
                    if let wast::Instruction::ReturnCall(index) = instr {
                        if let wast::Index::Num(num, _) = index.0.unwrap_index() {
                            let value = &branches[branches.len() - 1 - *num as usize];
                            let strict_len = value.strict_len();

                            let arg_stack = (strict_len..strict_len + args.len())
                                .map(Simple::Strict)
                                .collect::<Vec<_>>();

                            // push the previous arguments to the stack
                            for i in 0..strict_len {
                                self.function.instruction(Instruction::LocalGet(i as u32));
                            }

                            // greedy evaluation of all new arguments, inner first
                            for arg in args.iter() {
                                assert!(self
                                    .build(&Signature {
                                        expression: arg.clone(),
                                        arg_stack: vec![],
                                    })
                                    .is_num());
                            }

                            let signature = Signature {
                                expression: value.clone(),
                                arg_stack,
                            };

                            let f_index = self
                                .compiler
                                .0
                                .iter()
                                .position(|entry| entry.signature == signature)
                                .unwrap_or_else(|| self.compiler.compile_new(&signature));

                            let call_string = format!("call {}", f_index);
                            let call_buf = ParseBuffer::new(&call_string).unwrap();
                            let call = parser::parse::<wast::Instruction>(&call_buf).unwrap();

                            let mut buf = vec![];
                            call.encode(&mut buf);
                            self.function.raw(buf);
                        }
                    } else {
                        let mut buf = vec![];
                        instr.encode(&mut buf);
                        self.function.raw(buf);
                    }
                });

                Kind::Num
            }
            Simple::Bot => panic!("evaluated bottom"),
        }
    }
}
