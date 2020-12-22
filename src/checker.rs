use std::collections::HashMap;

use volpe_parser::ast::{BoolOp, IntOp, Op};
use z3::{
    ast::{Ast, Bool, BV},
    SatResult, Solver,
};

use crate::{
    core::CoreTerm,
    types::{Func, Type},
};

fn walk<'ctx>(
    tree: &CoreTerm,
    solver: &Solver<'ctx>,
    scope: &HashMap<String, Type<'ctx>>,
) -> Result<Type<'ctx>, String> {
    let ctx = solver.get_context();
    match tree {
        CoreTerm::Num(num) => Ok(BV::from_u64(ctx, *num, 64).into()),
        CoreTerm::Ident(name) => scope
            .get(name)
            .cloned()
            .ok_or_else(|| "ident not in scope".to_string()),
        CoreTerm::Unreachable => Err("reached unreachable!".to_string()),
        CoreTerm::Ite {
            cond,
            then,
            otherwise,
        } => {
            let cond = walk(cond, solver, scope)?
                .as_bool()
                .ok_or_else(|| "can only have condition on bool".to_string())?;
            match solver.check_assumptions(&[cond.clone()]) {
                SatResult::Unsat => return walk(otherwise, solver, scope),
                SatResult::Unknown => return Err("couldn\'t check!".to_string()),
                SatResult::Sat => {}
            }
            match solver.check_assumptions(&[cond.not()]) {
                SatResult::Unsat => return walk(then, solver, scope),
                SatResult::Unknown => return Err("couldn\'t check!".to_string()),
                SatResult::Sat => {}
            };
            solver.push();
            solver.assert(&cond);
            let then = walk(then, solver, scope);
            solver.pop(1);
            solver.push();
            let otherwise = walk(otherwise, solver, scope);
            solver.pop(1);
            Type::ite(&cond, &then?, &otherwise?)
                .ok_or_else(|| "types not the same in ite".to_string())
        }
        CoreTerm::Op { left, op, right } => Ok(match op {
            Op::Int(op) => {
                let left = walk(left, solver, scope)?
                    .as_bv()
                    .ok_or_else(|| "op only works for ints".to_string())?;
                let right = walk(right, solver, scope)?
                    .as_bv()
                    .ok_or_else(|| "op only works for ints".to_string())?;

                match op {
                    IntOp::Equal => left._eq(&right).into(),
                    IntOp::Unequal => left._eq(&right).not().into(),
                    IntOp::Less => left.bvslt(&right).into(),
                    IntOp::Greater => left.bvsgt(&right).into(),
                    IntOp::LessEqual => left.bvsle(&right).into(),
                    IntOp::GreaterEqual => left.bvsge(&right).into(),
                    IntOp::Add => left.bvadd(&right).into(),
                    IntOp::Sub => left.bvsub(&right).into(),
                    IntOp::Mul => left.bvmul(&right).into(),
                    IntOp::Div => left.bvsdiv(&right).into(),
                    IntOp::Mod => left.bvsmod(&right).into(),
                    IntOp::BitOr => left.bvor(&right).into(),
                    IntOp::BitAnd => left.bvand(&right).into(),
                    IntOp::BitXor => left.bvxor(&right).into(),
                    IntOp::BitShl => left.bvshl(&right).into(),
                    IntOp::BitShr => left.bvlshr(&right).into(),
                }
            }
            Op::Bool(op) => {
                let left = walk(left, solver, scope)?
                    .as_bool()
                    .ok_or_else(|| "op only works for bools".to_string())?;
                let right = walk(right, solver, scope)?
                    .as_bool()
                    .ok_or_else(|| "op only works for bools".to_string())?;

                match op {
                    BoolOp::And => Bool::and(ctx, &[&left, &right]),
                    BoolOp::Or => Bool::or(ctx, &[&left, &right]),
                }
                .into()
            }
            Op::Func => Type::Func(vec![(
                Bool::from_bool(ctx, true),
                Func {
                    arg: left.as_ref().clone(),
                    body: right.as_ref().clone(),
                    scope: scope.clone(),
                },
            )]),
            Op::App => {
                let left = walk(left, solver, scope)?
                    .as_func()
                    .ok_or_else(|| "can only call func".to_string())?;
                let val = walk(right, solver, scope)?;
                let ((_, prev), others) = left.split_last().unwrap();
                let mut new_scope = prev.scope.clone();
                assign(&mut new_scope, &prev.arg, &val);
                let mut prev = walk(&prev.body, solver, &new_scope)?;
                for (cond, f) in others.iter().rev() {
                    let mut new_scope = f.scope.clone();
                    assign(&mut new_scope, &f.arg, &val);
                    let val = walk(&f.body, solver, &new_scope)?;
                    prev = Type::ite(cond, &val, &prev)
                        .ok_or_else(|| "types not the same in ite".to_string())?;
                }
                prev
            }
        }),
        _ => unimplemented!(),
    }
}

fn assign<'ctx>(scope: &mut HashMap<String, Type<'ctx>>, arg: &CoreTerm, val: &Type<'ctx>) {
    let _ = match arg {
        CoreTerm::Ident(name) => scope.insert(name.clone(), val.clone()),
        _ => None,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use volpe_parser::parser::ExprParser;
    use z3::{Config, Context};

    macro_rules! check {
        ($s:literal, $solver:ident) => {
            walk(
                &(&ExprParser::new().parse($s).unwrap()).into(),
                &$solver,
                &HashMap::new(),
            )
        };
    }

    #[test]
    fn simple_num() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let s = Solver::new(&ctx);

        assert!(check!("1 == 1 => 0", s).is_ok());
        assert!(check!("1 == 3 => 0", s).is_err());
        assert!(check!("1 == 3 => 0; 2", s).is_ok());
        assert!(check!("a = 2; a == 2 => {}", s).is_ok());
        assert!(check!("a = 2; a == 3 => {}", s).is_err());
        assert!(dbg!(check!("f = x.x + 1; (f 1) == 2 => {}", s)).is_ok());
    }
}
