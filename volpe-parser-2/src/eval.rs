use crate::ast::{replace_simple, Simple};

#[derive(Default)]
pub struct Evaluator {
    buffer: String,
    args: Vec<Vec<Simple>>,
    prog: Vec<Simple>,
}

impl Evaluator {
    pub fn eval(ast: Vec<Simple>) -> String {
        let mut eval = Evaluator {
            buffer: String::new(),
            args: vec![],
            prog: ast,
        };
        while let Some(ast) = eval.prog.pop() {
            eval.eval_single(ast);
        }
        eval.buffer
    }

    pub fn eval_single(&mut self, ast: Simple) {
        match ast {
            Simple::Push(inner) => self.args.push(inner),
            Simple::Pop(names) => {
                for name in names {
                    let val = self.args.pop().unwrap();
                    replace_simple(&mut self.prog, name, &val);
                }
            }
            Simple::Ident(name) => {
                panic!("undefined {}", name)
            }
            Simple::Raw(text) => self.buffer.push_str(&text),
        }
    }
}
