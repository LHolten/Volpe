use volpe_parser::ast::{MultiOpCode, Term};
use z3::{
    ast::{Ast, Bool, Dynamic, BV},
    Context, SatResult, Solver,
};

fn walk<'ctx>(tree: &Term, ctx: &'ctx Context) -> Result<Dynamic<'ctx>, String> {
    match tree {
        Term::Num(num) => Ok(BV::from_u64(ctx, *num, 64).into()),
        Term::Assert { cond, val } => {
            let cond = walk(cond, ctx)?
                .as_bool()
                .ok_or(format!("can only have assert on bool"))?;
            let solver = Solver::new(ctx);
            match solver.check_assumptions(&[cond.not()]) {
                SatResult::Unsat => walk(val, ctx),
                SatResult::Unknown => Err(format!("couldn't check!")),
                SatResult::Sat => Err(format!("assertion error!")),
            }
        }
        Term::Ite {
            cond,
            then,
            otherwise,
        } => {
            let cond = walk(cond, ctx)?
                .as_bool()
                .ok_or(format!("can only have condition on bool"))?;
            let then = walk(then, ctx)?;
            let otherwise = walk(otherwise, ctx)?;
            // should check that `then` and `otherwise` are of the same type
            Ok(cond.ite(&then, &otherwise))
        }
        Term::MultiOp { head, tail } => {
            let mut prev = walk(head, ctx)?;
            let mut result = Bool::from_bool(ctx, true);
            for (op, next) in tail {
                let next = walk(next, ctx)?;
                // should check that `prev` and `next` are of the same type
                let cmp = match op {
                    MultiOpCode::Equal => prev._eq(&next),
                    _ => unimplemented!(),
                };
                prev = next;
                result = Bool::and(ctx, &[&result, &cmp]);
            }
            Ok(result.into())
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use volpe_parser::parser::ExprParser;
    use z3::Config;

    #[test]
    fn simple_num() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        assert!(walk(&ExprParser::new().parse("1 == 1 => 0").unwrap(), &ctx).is_ok());
        assert!(walk(&ExprParser::new().parse("1 == 1 => 0; 1").unwrap(), &ctx).is_ok());
    }
}
