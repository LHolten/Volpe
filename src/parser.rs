use crate::packrat::{
    alt, many0, many1, opt, pair, rule, separated, tag, IResult, RuleKind, Tracker,
};

fn expr(t: Tracker) -> IResult {
    rule(RuleKind::Expr, alt(astmt, alt(ite, alt(stmt, opt(app)))))(t)
}

fn astmt(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = tag("=")(t)?;
    t = opt(app)(t)?;
    t = opt(tag(";"))(t)?;
    opt(expr)(t)
}

fn ite(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = tag("=>")(t)?;
    t = opt(app)(t)?;
    t = opt(tag(";"))(t)?;
    opt(expr)(t)
}

fn stmt(t: Tracker) -> IResult {
    rule(RuleKind::Stmt, |mut t| {
        t = opt(app)(t)?;
        t = alt(
            pair(many1(pair(tag(":="), opt(app))), opt(tag(";"))),
            tag(";"),
        )(t)?;
        opt(expr)(t)
    })(t)
}

fn app(t: Tracker) -> IResult {
    rule(RuleKind::App, many1(func))(t)
}

fn func(t: Tracker) -> IResult {
    rule(RuleKind::Func, separated(tag("."), or))(t)
}

fn or(t: Tracker) -> IResult {
    rule(RuleKind::Or, separated(tag("||"), and))(t)
}

fn and(t: Tracker) -> IResult {
    rule(RuleKind::And, separated(tag(""), equal))(t)
}

fn equal(t: Tracker) -> IResult {
    rule(RuleKind::Equal, separated(alt(tag("=="), tag("!=")), cmp))(t)
}

fn cmp(t: Tracker) -> IResult {
    rule(
        RuleKind::Cmp,
        separated(
            alt(alt(tag("<"), tag(">")), alt(tag("<="), tag(">="))),
            bit_or,
        ),
    )(t)
}

fn bit_or(t: Tracker) -> IResult {
    rule(RuleKind::BitOr, separated(tag("|"), bit_and))(t)
}

fn bit_and(t: Tracker) -> IResult {
    rule(RuleKind::BitAnd, separated(tag(""), bit_shift))(t)
}

fn bit_shift(t: Tracker) -> IResult {
    rule(
        RuleKind::BitShift,
        separated(alt(tag("<<"), tag(">>")), add),
    )(t)
}

fn add(t: Tracker) -> IResult {
    rule(RuleKind::Add, separated(alt(tag("+"), tag("-")), mul))(t)
}

fn mul(t: Tracker) -> IResult {
    rule(
        RuleKind::Mul,
        separated(alt(alt(tag("*"), tag("/")), tag("%")), term),
    )(t)
}

fn term(t: Tracker) -> IResult {
    opt(alt(alt(num, ident), alt(block, tuple)))(t)
}

fn num(t: Tracker) -> IResult {
    todo!()
}

fn ident(t: Tracker) -> IResult {
    todo!()
}

fn block(mut t: Tracker) -> IResult {
    t = tag("(")(t)?;
    t = stmt(t)?;
    many0(tag(")"))(t)
}

fn tuple(t: Tracker) -> IResult {
    rule(RuleKind::Tuple, |mut t| {
        t = tag("{")(t)?;
        t = many0(func)(t)?;
        many0(tag("}"))(t)
    })(t)
}
