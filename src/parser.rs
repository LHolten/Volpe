use crate::packrat::{alt, many0, many1, opt, pair, separated, tag, IResult, Tracker};

fn expr(t: Tracker) -> IResult {
    alt(astmt, &alt(ite, &alt(stmt, &opt(app))))(t)
}

fn astmt(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = tag("=")(t)?;
    t = opt(app)(t)?;
    t = opt(&tag(";"))(t)?;
    opt(expr)(t)
}

fn ite(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = tag("=>")(t)?;
    t = opt(app)(t)?;
    t = opt(&tag(";"))(t)?;
    opt(expr)(t)
}

fn stmt(mut t: Tracker) -> IResult {
    t = opt(app)(t)?;
    t = alt(
        &pair(&many1(&pair(&tag("="), &opt(app))), &opt(&tag(";"))),
        &tag(";"),
    )(t)?;
    opt(expr)(t)
}

fn app(t: Tracker) -> IResult {
    many1(func)(t)
}

fn func(t: Tracker) -> IResult {
    separated(&tag("."), or)(t)
}

fn or(t: Tracker) -> IResult {
    separated(&tag("||"), and)(t)
}

fn and(t: Tracker) -> IResult {
    separated(&tag("&&"), equal)(t)
}

fn equal(t: Tracker) -> IResult {
    separated(&alt(&tag("=="), &tag("!=")), cmp)(t)
}

fn cmp(t: Tracker) -> IResult {
    separated(
        &alt(&alt(&tag("<"), &tag(">")), &alt(&tag("<="), &tag(">="))),
        bit_or,
    )(t)
}

fn bit_or(t: Tracker) -> IResult {
    separated(&tag("|"), bit_and)(t)
}

fn bit_and(t: Tracker) -> IResult {
    separated(&tag("&"), bit_shift)(t)
}

fn bit_shift(t: Tracker) -> IResult {
    separated(&alt(&tag("<<"), &tag(">>")), add)(t)
}

fn add(t: Tracker) -> IResult {
    separated(&alt(&tag("+"), &tag("-")), mul)(t)
}

fn mul(t: Tracker) -> IResult {
    separated(&alt(&alt(&tag("*"), &tag("/")), &tag("%")), term)(t)
}

fn term(t: Tracker) -> IResult {
    opt(&alt(&alt(num, ident), &alt(block, tuple)))(t)
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
    many0(&tag(")"))(t)
}

fn tuple(mut t: Tracker) -> IResult {
    t = tag("{")(t)?;
    t = many0(func)(t)?;
    many0(&tag("}"))(t)
}
