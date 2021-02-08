use crate::{
    lexer::Lexem as L,
    packrat::{alt, many0, many1, opt, pair, rule, separated, tag, IResult, RuleKind, Tracker},
};

pub fn expr(t: Tracker) -> IResult {
    rule(RuleKind::Expr, alt(astmt, alt(ite, alt(stmt, opt(app)))))(t)
}

fn astmt(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = tag(L::Assign)(t)?;
    t = opt(app)(t)?;
    t = opt(tag(L::NewLine))(t)?;
    opt(expr)(t)
}

fn ite(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = tag(L::Ite)(t)?;
    t = opt(app)(t)?;
    t = opt(tag(L::NewLine))(t)?;
    opt(expr)(t)
}

fn stmt(t: Tracker) -> IResult {
    rule(RuleKind::Stmt, |mut t| {
        t = opt(app)(t)?;
        t = alt(
            pair(
                many1(pair(tag(L::MultiAssign), opt(app))),
                opt(tag(L::NewLine)),
            ),
            tag(L::NewLine),
        )(t)?;
        opt(expr)(t)
    })(t)
}

fn app(t: Tracker) -> IResult {
    rule(RuleKind::App, many1(func))(t)
}

fn func(t: Tracker) -> IResult {
    rule(RuleKind::Func, separated(tag(L::Func), or))(t)
}

fn or(t: Tracker) -> IResult {
    rule(RuleKind::Or, separated(tag(L::Or), and))(t)
}

fn and(t: Tracker) -> IResult {
    rule(RuleKind::And, separated(tag(L::And), op1))(t)
}

fn op1(t: Tracker) -> IResult {
    rule(
        RuleKind::Op1,
        separated(
            alt(
                alt(tag(L::Equals), tag(L::UnEquals)),
                alt(
                    alt(tag(L::Less), tag(L::Greater)),
                    alt(tag(L::GreaterEqual), tag(L::LessEqual)),
                ),
            ),
            op2,
        ),
    )(t)
}

fn op2(t: Tracker) -> IResult {
    rule(
        RuleKind::Op2,
        separated(
            alt(
                alt(tag(L::Plus), tag(L::Minus)),
                alt(tag(L::BitOr), alt(tag(L::BitShl), tag(L::BitShr))),
            ),
            op3,
        ),
    )(t)
}

fn op3(t: Tracker) -> IResult {
    rule(
        RuleKind::Op3,
        separated(
            alt(
                alt(tag(L::Mul), tag(L::Div)),
                alt(tag(L::Mod), tag(L::BitAnd)),
            ),
            term,
        ),
    )(t)
}

fn term(t: Tracker) -> IResult {
    opt(alt(alt(tag(L::Num), tag(L::Ident)), alt(block, tuple)))(t)
}

fn block(mut t: Tracker) -> IResult {
    t = tag(L::LBrace)(t)?;
    t = stmt(t)?;
    opt(tag(L::RBrace))(t)
}

fn tuple(t: Tracker) -> IResult {
    rule(RuleKind::Tuple, |mut t| {
        t = tag(L::LCurlyBrace)(t)?;
        t = many0(func)(t)?;
        opt(tag(L::RCurlyBrace))(t)
    })(t)
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
