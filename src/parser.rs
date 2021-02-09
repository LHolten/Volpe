use crate::{
    lexer::Lexem as L,
    packrat::{alt, many0, many1, opt, pair, rule, separated, tag, IResult, RuleKind, Tracker},
};

pub fn expr(t: Tracker) -> IResult {
    rule(RuleKind::Expr, pair(app, opt(alt(astmt, alt(ite, stmt)))))(t)
}

fn astmt(mut t: Tracker) -> IResult {
    t = tag(L::Assign)(t)?;
    t = app(t)?;
    t = opt(tag(L::NewLine))(t)?;
    expr(t)
}

fn ite(mut t: Tracker) -> IResult {
    t = tag(L::Ite)(t)?;
    t = app(t)?;
    t = opt(tag(L::NewLine))(t)?;
    expr(t)
}

fn stmt(t: Tracker) -> IResult {
    rule(RuleKind::Stmt, |mut t| {
        t = alt(
            pair(many1(pair(tag(L::MultiAssign), app)), opt(tag(L::NewLine))),
            tag(L::NewLine),
        )(t)?;
        expr(t)
    })(t)
}

fn app(t: Tracker) -> IResult {
    rule(RuleKind::App, many1(pair(term, func)))(t)
}

fn func(t: Tracker) -> IResult {
    alt(rule(RuleKind::Func, many1(pair(tag(L::Func), or))), or)(t)
}

fn or(t: Tracker) -> IResult {
    alt(rule(RuleKind::Or, many1(pair(tag(L::Or), and))), and)(t)
}

fn and(t: Tracker) -> IResult {
    alt(rule(RuleKind::And, many1(pair(tag(L::And), op1))), op1)(t)
}

fn op1(t: Tracker) -> IResult {
    alt(
        rule(
            RuleKind::Op1,
            many1(pair(
                tag(L::Equals | L::UnEquals | L::Less | L::Greater | {
                    L::GreaterEqual | L::LessEqual
                }),
                op2,
            )),
        ),
        op2,
    )(t)
}

fn op2(t: Tracker) -> IResult {
    alt(
        rule(
            RuleKind::Op2,
            many1(pair(
                tag(L::Plus | L::Minus | L::BitOr | L::BitShl | L::BitShr),
                op3,
            )),
        ),
        op3,
    )(t)
}

fn op3(t: Tracker) -> IResult {
    alt(
        rule(
            RuleKind::Op3,
            many1(pair(tag(L::Mul | L::Div | L::Mod | L::BitAnd), term)),
        ),
        term,
    )(t)
}

fn term(t: Tracker) -> IResult {
    opt(alt(tag(L::Num | L::Ident), alt(block, tuple)))(t)
}

fn block(mut t: Tracker) -> IResult {
    t = tag(L::LBrace)(t)?;
    t = stmt(t)?;
    // t = many0(tag(L::RCurlyBrace))(t)?;
    opt(tag(L::RBrace))(t)
}

fn tuple(mut t: Tracker) -> IResult {
    t = tag(L::LCurlyBrace)(t)?;
    t = app(t)?;
    // t = many0(tag(L::LBrace))(t)?;
    opt(tag(L::RCurlyBrace))(t)
}

#[cfg(test)]
mod tests {
    use crate::packrat::SharedPosition;

    use super::expr;

    macro_rules! test_expr {
        ($s:literal) => {
            let pos = SharedPosition::new();
            pos.patch(None, $s, 0, 0).unwrap_err();
            assert!(pos.parse(expr).is_ok())
        };
    }

    #[test]
    fn functions() {
        test_expr!("hello world");
        // test_expr!("[1, 2, 3; 4, 5, 6] 10 (cool thing)");
        test_expr!("(a = 1; b = 2; add a b)");
        // test_expr!(
        //     "my_object = {
        //         alpha : something,
        //         beta : 3404,
        //     }; my_object"
        // );
        test_expr!("a.b.(add a b) 10 20");
        test_expr!("{1, 2, 3}");
        test_expr!("{1}");
        test_expr!("{}");
        test_expr!("1 > 2 => {}");
        // test_expr!("1 /* /* wow */ cool */ > 2 // hello");
    }
}
