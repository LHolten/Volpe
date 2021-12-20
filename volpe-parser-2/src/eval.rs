use crate::{offset::Range, simple::Simple};

#[derive(Default)]
pub struct Evaluator<'a> {
    buffer: String,
    args: Vec<Vec<Simple<'a>>>,
    prog: Vec<Simple<'a>>,
    pub refs: Vec<Reference<'a>>,
}

impl<'a> Evaluator<'a> {
    pub fn eval(ast: Vec<Simple>) -> String {
        let mut eval = Evaluator {
            buffer: String::new(),
            args: vec![],
            prog: ast,
            refs: vec![],
        };
        while let Some(ast) = eval.prog.pop() {
            eval.eval_single(ast);
        }
        eval.buffer
    }

    pub fn eval_single(&mut self, ast: Simple<'a>) {
        match ast {
            Simple::Push(inner) => self.args.push(inner),
            Simple::Pop(name) => {
                let val = self.args.pop().unwrap();
                replace_simple(&mut self.prog, name, &val);
            }
            Simple::Ident(name) => {
                panic!("undefined {}", name.text)
            }
            Simple::Raw(raw) => self.buffer.push_str(raw.text),
        }
    }
}

pub struct Reference<'a> {
    pub from: Range<'a>,
    pub to: Range<'a>,
}

pub fn replace_simple<'a>(
    list: &mut Vec<Simple<'a>>,
    name: Range<'a>,
    val: &[Simple<'a>],
) -> Vec<Reference<'a>> {
    let mut refs = vec![];
    for i in (0..list.len()).rev() {
        match &mut list[i] {
            Simple::Push(inner) => refs.extend(replace_simple(inner, name, val)),
            Simple::Pop(n) => {
                if n.text == name.text {
                    return refs;
                }
            }
            Simple::Ident(n) => {
                if n.text == name.text {
                    refs.push(Reference { from: *n, to: name });
                    list.splice(i..i + 1, val.iter().cloned());
                }
            }
            _ => {}
        }
    }
    refs
}
