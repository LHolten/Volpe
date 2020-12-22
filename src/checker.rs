use volpe_parser::ast::{BoolOp, IntOp, Op};
use z3::{
    ast::{Ast, Bool, Dynamic, BV},
    SatResult, Solver,
};

use crate::core::CoreTerm;

fn walk<'ctx>(tree: &CoreTerm, solver: &Solver<'ctx>) -> Result<Dynamic<'ctx>, String> {
    let ctx = solver.get_context();
    match tree {
        CoreTerm::Num(num) => Ok(BV::from_u64(ctx, *num, 64).into()),
        CoreTerm::Unreachable => Err("reached unreachable!".to_string()),
        CoreTerm::Ite {
            cond,
            then,
            otherwise,
        } => {
            let cond = walk(cond, solver)?
                .as_bool()
                .ok_or_else(|| "can only have condition on bool".to_string())?;
            let then = walk(then, solver)?;
            match solver.check_assumptions(&[cond.not()]) {
                SatResult::Unsat => Ok(then),
                SatResult::Unknown => Err("couldn\'t check!".to_string()),
                SatResult::Sat => {
                    let otherwise = walk(otherwise, solver)?;
                    // should check that `then` and `otherwise` are of the same type
                    Ok(cond.ite(&then, &otherwise))
                }
            }
        }
        CoreTerm::Op { left, op, right } => {
            let left = walk(left, solver)?;
            let right = walk(right, solver)?;
            Ok(match op {
                Op::Int(op) => {
                    let left = left
                        .as_bv()
                        .ok_or_else(|| "op only works for ints".to_string())?;
                    let right = right
                        .as_bv()
                        .ok_or_else(|| "op only works for ints".to_string())?;

                    match op {
                        IntOp::Equal => left._eq(&right),
                        IntOp::Unequal => left._eq(&right).not(),
                        IntOp::Less => left.bvslt(&right),
                        IntOp::Greater => left.bvsgt(&right),
                        IntOp::LessEqual => left.bvsle(&right),
                        IntOp::GreaterEqual => left.bvsge(&right),
                        _ => unimplemented!(),
                    }
                    .into()
                }
                Op::Bool(op) => {
                    let left = left
                        .as_bool()
                        .ok_or_else(|| "op only works for bools".to_string())?;
                    let right = right
                        .as_bool()
                        .ok_or_else(|| "op only works for bools".to_string())?;

                    match op {
                        BoolOp::And => Bool::and(ctx, &[&left, &right]),
                        BoolOp::Or => Bool::or(ctx, &[&left, &right]),
                    }
                    .into()
                }
                _ => unimplemented!(),
            })
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use volpe_parser::parser::ExprParser;
    use z3::{Config, Context};

    #[test]
    fn simple_num() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        assert!(walk(
            &(&ExprParser::new().parse("1 == 1 => 0").unwrap()).into(),
            &solver
        )
        .is_ok());
        assert!(walk(
            &(&ExprParser::new().parse("1 == 1 => 0; 1").unwrap()).into(),
            &solver
        )
        .is_ok());
    }
}
