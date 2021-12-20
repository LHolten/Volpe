use std::mem::replace;

use crate::{offset::Range, simple::Simple};

#[derive(Default)]
pub struct Evaluator<'a> {
    buffer: String,
    args: Vec<Simple<'a>>,
    prog: Vec<Simple<'a>>,
    pub refs: Vec<Reference<'a>>,
}

impl<'a> Evaluator<'a> {
    pub fn eval(ast: Vec<Simple>) -> Result<String, String> {
        let mut eval = Evaluator {
            buffer: String::new(),
            args: vec![],
            prog: ast,
            refs: vec![],
        };
        while let Some(ast) = eval.prog.pop() {
            eval.eval_single(ast)?;
        }
        Ok(eval.buffer)
    }

    pub fn eval_single(&mut self, ast: Simple<'a>) -> Result<(), String> {
        match ast {
            Simple::Push(inner) => self.args.push(*inner),
            Simple::Pop(name) => {
                let val = self.args.pop().unwrap();
                self.refs.extend(replace_simple(&mut self.prog, name, &val))
            }
            Simple::Ident(name) => return Err(name.text.to_string()),
            Simple::Raw(raw) => self.buffer.push_str(raw.text),
            Simple::Scope(inner) => {
                let temp = replace(&mut self.prog, inner);
                while let Some(ast) = self.prog.pop() {
                    self.eval_single(ast)?;
                }
                self.prog = temp;
            }
        }
        Ok(())
    }
}

pub struct Reference<'a> {
    pub from: Range<'a>,
    pub to: Range<'a>,
}

pub fn replace_simple<'a>(
    list: &mut Vec<Simple<'a>>,
    name: Range<'a>,
    val: &Simple<'a>,
) -> Vec<Reference<'a>> {
    let mut refs = vec![];
    for i in (0..list.len()).rev() {
        let mut item = &mut list[i];
        loop {
            match item {
                Simple::Push(inner) => {
                    item = inner.as_mut();
                    continue;
                }
                Simple::Pop(n) => {
                    if n.text == name.text {
                        return refs;
                    }
                }
                Simple::Ident(n) => {
                    if n.text == name.text {
                        refs.push(Reference { from: *n, to: name });
                        list[i] = val.clone()
                    }
                }
                Simple::Scope(inner) => refs.extend(replace_simple(inner, name, val)),
                Simple::Raw(_) => {}
            }
            break;
        }
    }
    refs
}
