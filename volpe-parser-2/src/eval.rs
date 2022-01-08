use crate::{offset::Range, simple::Simple};

#[derive(Default)]
pub struct Evaluator<'a> {
    buffer: String,
    args: Vec<Simple<'a>>,
    pub refs: Vec<Reference<'a>>,
}

impl<'a> Evaluator<'a> {
    pub fn eval(mut ast: Vec<Simple>) -> Result<String, String> {
        let mut eval = Evaluator {
            buffer: String::new(),
            args: vec![],
            refs: vec![],
        };
        eval.eval_single(&mut ast)?;
        Ok(eval.buffer)
    }

    pub fn eval_single(&mut self, ast: &mut Vec<Simple<'a>>) -> Result<(), String> {
        let last = match ast.pop() {
            Some(ast) => ast,
            None => return Ok(()),
        };
        match last {
            Simple::Push(inner) => self.args.push(*inner),
            Simple::Pop(name) => {
                let err = format!("no arg for: {}", name.text);
                let val = self.args.pop().ok_or(err)?;
                self.refs.extend(replace_simple(ast, name, &val))
            }
            Simple::Ident(name) => return Err(name.text.to_string()),
            Simple::Raw(raw) => self.buffer.push_str(raw.text),
            Simple::Scope(mut inner) => {
                self.eval_single(&mut inner)?;
            }
        }
        self.eval_single(ast)
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

#[cfg(test)]
mod tests {
    use crate::{eval::Evaluator, file::File, offset::Offset};

    fn check(input: &str, output: Result<String, String>) {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), input.to_string())
            .unwrap();
        let syntax = file.rule().collect().unwrap();
        let result = Evaluator::eval(syntax.convert(None));
        assert_eq!(result, output);
    }

    #[test]
    fn resolution1() {
        check(
            "
            [a] (b)
            [b] (1)
            a",
            Ok("1".to_string()),
        )
    }
    #[test]
    fn resolution2() {
        check(
            "
            [b] (1)
            [a] (b)
            a",
            Ok("1".to_string()),
        )
    }
    #[test]
    fn resolution3() {
        check(
            "
            [f] (
                [a] (1)
                {}
            )
            f(a)",
            Err("a".to_string()),
        )
    }

    #[test]
    fn resolution4() {
        check(
            "
            [a] (1)
            [a] (2)
            a",
            Ok("2".to_string()),
        )
    }

    #[test]
    fn resolution5() {
        check(
            "
            {} ([a] (1))
            a",
            Err("a".to_string()),
        )
    }

    #[test]
    fn other() {
        check(
            r##"
        [new] ( [arg]
            arg((attr))
        )

        [obj] new {
            [attr] 2
        }

        [attr] 1
        obj
        "##,
            Ok("2".to_string()),
        );
    }
}
