use std::{cell::Cell, collections::HashMap};

use typed_arena::Arena;
use volpe_parser::ast::{BoolOp, IntOp, Op};
use z3::{
    ast::{Ast, Bool, BV},
    Config, Context, SatResult, Solver,
};

use crate::{
    core::CoreTerm,
    types::Type,
    wlp::{IteWLP, OpWLP, WLP},
};

fn walk<'ctx>(term: &Cell<WLP>, ctx: &'ctx Context) -> Result<Type<'ctx>, String> {
    Ok(match term.get() {
        WLP::Num(num) => BV::from_u64(ctx, num, 64).into(),
        WLP::Bool(val) => Bool::from_bool(ctx, val).into(),
        WLP::Unknown(name) => BV::new_const(ctx, name, 64).into(),
        WLP::Op(OpWLP { left, op, right }) => match op {
            Op::Int(op) => {
                let left = walk(left, ctx)?.as_bv().ok_or("op only works for ints")?;
                let right = walk(right, ctx)?.as_bv().ok_or("op only works for ints")?;

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
                let left = walk(left, ctx)?
                    .as_bool()
                    .ok_or("op only works for bools")?;
                let right = walk(right, ctx)?
                    .as_bool()
                    .ok_or("op only works for bools")?;

                match op {
                    BoolOp::And => Bool::and(ctx, &[&left, &right]),
                    BoolOp::Or => Bool::or(ctx, &[&left, &right]),
                }
                .into()
            }
            _ => unimplemented!(),
        },
        WLP::Unfinished => BV::fresh_const(ctx, "unf", 64).into(),
        WLP::Ite(IteWLP {
            cond,
            then,
            otherwise,
        }) => Type::ite(
            &walk(cond, ctx)?
                .as_bool()
                .ok_or("can only have condition on bool")?,
            &walk(then, ctx)?,
            &walk(otherwise, ctx)?,
        )
        .ok_or("incompatible types in ite")?,
    })
}

fn wlp<'a>(
    term: &'a CoreTerm,
    post: &mut Vec<&'a Cell<WLP<'a>>>,
    scope: &HashMap<String, &'a CoreTerm>,
    args: &mut Vec<&'a CoreTerm>,
    arena: &'a Arena<WLP<'a>>,
) -> Result<(), String> {
    let last = post.last().unwrap();
    // result is bool
    match term {
        CoreTerm::Unreachable => {
            last.replace(WLP::Bool(false));
        }
        CoreTerm::Num(val) => {
            last.replace(WLP::Num(*val));
        }
        CoreTerm::Ident(name) => {
            if let Some(val) = scope.get(name) {
                wlp(val, post, scope, args, arena)?
            } else {
                last.replace(WLP::Unknown(name.as_str()));
            }
        }
        CoreTerm::Op {
            left: func,
            op: Op::App,
            right: val,
        } => {
            args.push(val.as_ref());
            wlp(func.as_ref(), post, scope, args, arena)?;
            args.pop().unwrap();
        }
        CoreTerm::Op {
            left: arg,
            op: Op::Func,
            right: body,
        } => {
            let mut new_args = args.clone();
            let val = new_args.pop().ok_or("can't return func")?;
            let mut new_scope = scope.clone();
            let _ = match arg.as_ref() {
                CoreTerm::Ident(name) => new_scope.insert(name.clone(), val),
                _ => None,
            };
            wlp(body.as_ref(), post, &new_scope, &mut new_args, arena)?
        }
        CoreTerm::Op { left, op, right } => {
            let val = OpWLP::new(*op, arena);
            last.replace(WLP::Op(val));
            post.push(val.left);
            wlp(left, post, scope, args, arena)?;
            *post.last_mut().unwrap() = val.right;
            wlp(right, post, scope, args, arena)?;
            post.pop().unwrap();
        }
        CoreTerm::Ite {
            cond,
            then,
            otherwise,
        } => {
            let val = IteWLP::new(arena);
            last.replace(WLP::Ite(val));
            post.push(val.then);
            wlp(then, post, scope, args, arena)?;
            *post.last_mut().unwrap() = val.otherwise;
            wlp(otherwise, post, scope, args, arena)?;
            *post.last_mut().unwrap() = val.cond;
            wlp(cond, post, scope, &mut Vec::new(), arena)?;
            post.pop().unwrap();
        }
        val => {
            dbg!(val);
            unimplemented!()
        }
    };
    Ok(())
}

fn prove(term: CoreTerm) -> Result<(), String> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    let scope = HashMap::new();
    let arena = Arena::new();
    let val = Cell::from_mut(arena.alloc(WLP::Unfinished));
    wlp(&term, &mut vec![val], &scope, &mut Vec::new(), &arena)?;
    let cond = walk(val, &ctx)?;
    let res = match solver.check_assumptions(&[cond.as_bool().unwrap().not()]) {
        SatResult::Unsat => Ok(()),
        SatResult::Unknown => Err("unknown".to_string()),
        SatResult::Sat => Err("counter example".to_string()),
    };
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use volpe_parser::parser::ExprParser;

    #[test]
    fn no_func() {
        let parser = ExprParser::new();
        prove(parser.parse("0 < 1").unwrap().into()).unwrap();
        prove(parser.parse("0 > 1").unwrap().into()).unwrap_err();
        prove(parser.parse("a == a").unwrap().into()).unwrap();
        prove(parser.parse("a == b").unwrap().into()).unwrap_err();
    }

    #[test]
    fn simple_func() {
        let parser = ExprParser::new();
        prove(parser.parse("{} a == a").unwrap().into()).unwrap();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use volpe_parser::parser::ExprParser;
//     use z3::{Config, Context};

//     macro_rules! check {
//         ($s:literal, $solver:ident) => {
//             walk(
//                 &(&ExprParser::new().parse($s).unwrap()).into(),
//                 &$solver,
//                 &HashMap::new(),
//             )
//         };
//     }

//     #[test]
//     fn simple_num() {
//         let cfg = Config::new();
//         let ctx = Context::new(&cfg);
//         let s = Solver::new(&ctx);

//         assert!(check!(
//             "
//             fix := f.(x.(f (x x)) x.(f (x x)));

//             func := fix func.a.(
//                 a == 0 => {};
//                 func a - 1
//             );

//             x < 0 || x > 10 => {};
//             func x
//         ",
//             s
//         )
//         .is_ok());
//         // assert!(check!("1 == 1 => 0", s).is_ok());
//         // assert!(check!("1 == 3 => 0", s).is_err());
//         // assert!(check!("1 == 3 => 0; 2", s).is_ok());
//         // assert!(check!("a := 2; a == 2 => {}", s).is_ok());
//         // assert!(check!("a := 2; a == 3 => {}", s).is_err());
//         // assert!(check!("f := x.x + 1; (f 1) == 2 => {}", s).is_ok());
//         // assert!(check!("1 < 2 < 3 => {}", s).is_ok());
//         // assert!(check!("a > b || a == b || a < b => {}", s).is_ok());
//         // assert!(check!("a > b || a < b => {}", s).is_err());
//         // assert!(check!("x := n % 2; x == 0 || x == 1 => {}", s).is_ok());
//         // assert!(check!(
//         //     "assume := cond.then.(cond => then; {});
//         //     assert := cond.(cond => {});

//         //     assume 0 <= x;
//         //     assume 0 <= y;
//         //     assume x / 2 == y / 2;
//         //     assert x == y || x + 1 == y || x == y + 1",
//         //     s
//         // )
//         // .is_ok());
//     }
// }
