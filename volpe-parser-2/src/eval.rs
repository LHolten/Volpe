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
                let err = format!("no arg for: {}", name.text);
                let val = self.args.pop().ok_or(err)?;
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

#[cfg(test)]
mod tests {
    use crate::{eval::Evaluator, file::File, offset::Offset};

    fn check(input: &str, output: Result<String, String>) {
        let mut file = File::default();
        file.patch(Offset::default(), Offset::default(), input.to_string())
            .unwrap();
        let syntax = file.rule().collect().unwrap();
        let result = Evaluator::eval(syntax.convert());
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
        [module] ( [main]
            #{(module}
                #{(func (export "main") (result }
                    main(type)
                #{)}
                main(locals)
                main(body)
                #{)}
            #{)}
        )
        
        [i32] ( [lit]
            {
                [type] (#{i32})
                [body] (
                    #{(i32.const }
                        lit
                    #{)}
                )
                [locals] ()
                [is_i32] ()
            }
        )
        
        [add_i32] ( [a b]
            a (is_i32)
            b (is_i32)
            {
                [type] (#{i32})
                [body] (#{(i32.add)} a(body) b(body))
                [locals] (a(locals) b(locals))
                [is_i32] ()
            }
        )
        
        [var] ( [expr]
            [ident] (#{$0})
            [cont]
            [res] (
                cont ({
                    [type] (expr(type))
                    [locals] (expr(locals))
                    [body] (
                        #{(get_local }
                            ident
                        #{)}
                    )
                })
            )
            {} ( [v]
                [type] (res(type))
                [body] (
                    #{(set_local } expr(body)
                        ident
                    #{)}
                    res(body)
                )
                [locals] (
                    res(locals)
                    #{(local }
                        ident
                    #{ }
                        expr(type)
                    #{)}
                )
                res(v)
            )
        )
        // add_i32(i32(1))(i32(1))
        module (
            var(i32(1)); [total]
            total
        )"##,
            Ok(Default::default()),
        );
    }
}
