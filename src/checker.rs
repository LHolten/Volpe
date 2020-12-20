use volpe_parser::ast::{MultiOpCode, Term};
use z3::{
    ast::{Ast, Bool, Dynamic, BV},
    Context, Solver,
};

fn walk<'ctx>(tree: &Term, ctx: &'ctx Context) -> Dynamic<'ctx> {
    match tree {
        Term::Num(num) => BV::from_u64(ctx, *num, 64).into(),
        Term::Assert { cond, val } => {
            let cond = walk(cond, ctx)
                .as_bool()
                .expect("can only have assert on bool");
            let val = walk(val, ctx);
            let solver = Solver::new(ctx);
            match solver.check_assumptions(&[cond.not()]) {
                z3::SatResult::Unsat => val,
                _ => panic!("assertion error!"),
            }
        }
        Term::Ite {
            cond,
            then,
            otherwise,
        } => {
            let cond = walk(cond, ctx)
                .as_bool()
                .expect("can only have condition on bool");
            let then = walk(then, ctx);
            let otherwise = walk(otherwise, ctx);
            cond.ite(&then, &otherwise)
        }
        Term::MultiOp { head, tail } => {
            let mut prev = walk(head, ctx);
            let mut result = Bool::from_bool(ctx, true);
            for (op, next) in tail {
                let next = walk(next, ctx);
                let cmp = match op {
                    MultiOpCode::Equal => prev._eq(&next),
                    _ => unimplemented!(),
                };
                prev = next;
                result = Bool::and(ctx, &[&result, &cmp]);
            }
            result.into()
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

        walk(&ExprParser::new().parse("1 == 1 => 0").unwrap(), &ctx);
    }
}
