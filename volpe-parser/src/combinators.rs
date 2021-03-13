use std::{cell::Cell, rc::Rc};

use crate::{
    offset::Offset,
    position::{Rule, RuleKind, Syntax},
    tracker::{IResult, Tracker},
    with_cell::WithInternal,
};

pub fn tag(kind: impl Into<usize> + Copy) -> impl Fn(Tracker) -> IResult {
    move |mut t: Tracker| {
        let length = t.pos.with(|p| p.length);
        t.update_length(length);
        if 1 << t.pos.with(|p| p.kind) as usize & kind.into() != 0 {
            t.add_child(Syntax::Lexem(t.pos.clone(), true));
            t.offset += length;
            t.pos = t.pos.with(|p| p.next.upgrade().unwrap());
            Ok(t)
        } else {
            t.add_child(Syntax::Lexem(t.pos.clone(), false));
            Err(())
        }
    }
}

pub fn rule(kind: RuleKind, f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |mut t: Tracker| {
        if let Some(rule) = t.pos.with(|p| p.rules[kind as usize].upgrade()) {
            rule.with(|r| t.update_length(r.length));
            let success = rule.with(|r| r.success.clone());
            t.add_child(Syntax::Rule(rule));
            if let Some((offset, pos, _)) = success {
                t.offset += offset;
                t.pos = pos;
                Ok(t)
            } else {
                Err(())
            }
        } else {
            let t2 = Tracker {
                pos: t.pos,
                offset: Offset::default(),
                children: &Default::default(),
                length: &Default::default(),
            };
            let success = f(t2.clone()).ok().map(|t3| (t3.offset, t3.pos, kind));
            let rule = Rc::new(Cell::new(Rule {
                length: t2.length.get(),
                children: t2.children.replace(Vec::new()),
                success: success.clone(),
            }));
            t.pos = t2.pos;
            t.pos
                .with(|p| p.rules[kind as usize] = Rc::downgrade(&rule));
            t.add_child(Syntax::Rule(rule));
            t.update_length(t2.length.get());
            if let Some((offset, pos, _)) = success {
                t.offset += offset;
                t.pos = pos;
                Ok(t)
            } else {
                Err(())
            }
        }
    }
}

pub fn separated(
    symbol: impl Fn(Tracker) -> IResult,
    next: impl Fn(Tracker) -> IResult,
) -> impl Fn(Tracker) -> IResult {
    move |t| many1(pair(&symbol, &next))(next(t)?)
}

pub fn many2(f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |t| {
        let new_t = f(t.clone())?;
        if new_t.offset == t.offset {
            Err(())
        } else {
            many1(&f)(new_t)
        }
    }
}

pub fn many1(f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |t| {
        let new_t = f(t.clone())?;
        if new_t.offset == t.offset {
            Err(())
        } else {
            many0(&f)(new_t)
        }
    }
}

pub fn many0(f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |mut t| {
        while let Ok(new_t) = f(t.clone()) {
            if t.offset == new_t.offset {
                return Ok(new_t);
            }
            t = new_t
        }
        Ok(t)
    }
}

pub fn pair(
    f: impl Fn(Tracker) -> IResult,
    g: impl Fn(Tracker) -> IResult,
) -> impl Fn(Tracker) -> IResult {
    move |t| g(f(t)?)
}

pub fn alt(
    f: impl Fn(Tracker) -> IResult,
    g: impl Fn(Tracker) -> IResult,
) -> impl Fn(Tracker) -> IResult {
    move |t| {
        if let Ok(t) = f(t.clone()) {
            Ok(t)
        } else {
            g(t)
        }
    }
}

pub fn opt(f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |t| {
        if let Ok(t) = f(t.clone()) {
            Ok(t)
        } else {
            Ok(t)
        }
    }
}
