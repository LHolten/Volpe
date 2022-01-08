use crate::{offset::Range, simple::Simple};

#[derive(Debug, Clone)]
struct Scope<'a> {
    val: Vec<Simple<'a>>,
    env: Vec<(Range<'a>, Scope<'a>)>,
}

#[derive(Default)]
pub struct Evaluator<'a> {
    buffer: String,
    args: Vec<Scope<'a>>,
    pub refs: Vec<Reference<'a>>,
}

impl<'a> Evaluator<'a> {
    pub fn eval(ast: Vec<Simple>) -> Result<String, String> {
        let mut eval = Evaluator {
            buffer: String::new(),
            args: vec![],
            refs: vec![],
        };
        eval.eval_single(Scope {
            val: ast,
            env: vec![],
        })?;
        if !eval.args.is_empty() {
            let err = format!("too many args: {}", eval.args.len());
            return Err(err);
        }
        Ok(eval.buffer)
    }

    fn eval_single(&mut self, mut scope: Scope<'a>) -> Result<(), String> {
        let last = match scope.val.pop() {
            Some(ast) => ast,
            None => return Ok(()),
        };
        match last {
            Simple::Push(inner) => self.args.push(Scope {
                val: inner,
                env: scope.env.clone(),
            }),
            Simple::Pop(name) => {
                let err = format!("no arg for: {}", name.text);
                let val = self.args.pop().ok_or(err)?;
                scope.env.push((name, val));
            }
            Simple::Ident(name) => {
                for (n, s) in scope.env.iter().rev() {
                    if n.text == name.text {
                        // TODO add to refs
                        let mut inner = s.clone();
                        // inner.env.splice(0..0, scope.env.clone());
                        inner.env.extend(scope.env.clone());
                        self.eval_single(inner)?;
                        return self.eval_single(scope);
                    }
                }
                return Err(name.text.to_string());
            }

            Simple::Raw(raw) => self.buffer.push_str(raw.text),
        }
        self.eval_single(scope)
    }
}

pub struct Reference<'a> {
    pub from: Range<'a>,
    pub to: Range<'a>,
}

#[cfg(test)]
mod tests {
    use crate::{eval::Evaluator, file::File, offset::Offset};

    fn check(input: &str, output: Result<&str, &str>) {
        let output = output.map(str::to_string).map_err(str::to_string);
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
            [b] 1
            a",
            Ok("1"),
        )
    }

    #[test]
    fn resolution2() {
        check(
            "
            [b] 1
            [a] (b)
            a",
            Ok("1"),
        )
    }

    #[test]
    fn resolution3() {
        check(
            "
            [f] (
                [a] 1
                v [v]
            )
            f(a)",
            Err("a"),
        )
    }

    #[test]
    fn resolution4() {
        check(
            "
            [a] 1
            [a] 2
            a",
            Ok("2"),
        )
    }

    #[test]
    fn resolution5() {
        check(
            "
            v [v] ([a] 1)
            a",
            Err("a"),
        )
    }

    #[test]
    fn resolution6() {
        check(
            "
            [x] (y)
            [v] { [x] 1 }
            v (x)",
            Ok("1"),
        )
    }

    #[test]
    fn resolution7() {
        check(
            "
            [y] (x)
            [v] { [x] 1 }
            v (y)",
            Ok("1"),
        )
    }

    #[test]
    fn other() {
        check(
            r##"
        [new] ( [arg]
            (arg(attr))
        )

        [obj] new {
            [attr] 2
        }

        [attr] 1
        obj
        "##,
            Ok("2"),
        );
    }
}
