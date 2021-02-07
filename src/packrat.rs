use std::{cell::Cell, rc::Rc};

pub type IResult<'t> = Result<Tracker<'t>, ()>;
pub trait Parser: Copy {
    fn parse(self, t: Tracker) -> IResult;
}

impl<F> Parser for F
where
    for<'r> F: Fn(Tracker<'r>) -> IResult<'r> + Copy,
{
    fn parse(self, t: Tracker) -> IResult {
        self(t)
    }
}

pub struct Rule {
    pub length: usize,
    pub kind: RuleKind,
    pub children: Vec<(Option<RuleKind>, Rc<Position>)>,
    pub next: Option<(usize, Rc<Position>)>,
}

pub struct Position {
    pub lexem: String, // white space in front
    pub rules: [Cell<Option<Rule>>; 7],
    pub next: Option<Rc<Position>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RuleKind {
    Ident,
    Num,
    Symbol,
    Unknown,
    OpAnd,
    OpOr,
    Operator,
    Function,
    Application,
}

impl Position {
    pub fn with_rule<R>(&self, kind: RuleKind, f: impl Fn(&Rule) -> R) -> Option<R> {
        let rule = self.rules[kind as usize].replace(None)?;
        let res = f(&rule);
        self.rules[kind as usize].set(Some(rule));
        Some(res)
    }

    pub fn update(&self, kind: RuleKind, rule: Rule) {
        self.rules[kind as usize].set(Some(rule))
    }
}

#[derive(Clone)]
pub struct Tracker<'t> {
    pub input: Rc<Position>,
    pub offset: usize,
    pub children: &'t Cell<Vec<(Option<RuleKind>, Rc<Position>)>>,
    pub length: &'t Cell<usize>,
}

impl<'t> Tracker<'t> {
    pub fn add_child(&self, kind: Option<RuleKind>, pos: Rc<Position>) {
        let mut children = self.children.replace(Vec::new());
        children.push((kind, pos));
        self.children.set(children)
    }

    pub fn update_length(&self, length: usize) {
        self.length
            .set((self.offset + length).max(self.length.get()));
    }
}

pub fn tag(lexem: &'static str) -> impl Fn(Tracker) -> IResult {
    move |mut t| {
        t.add_child(None, t.input.clone());
        t.update_length(lexem.len());
        if t.input.lexem == lexem {
            t.offset += lexem.len();
            t.input = t.input.next.clone().unwrap();
            Ok(t)
        } else {
            Err(())
        }
    }
}

pub fn rule(kind: RuleKind, f: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |mut t| {
        t.add_child(Some(kind), t.input.clone());
        if let Some(res) = t.input.with_rule(kind, |r| {
            if let Some(next) = r.next.clone() {
                (r.length, Some(next))
            } else {
                (r.length, None)
            }
        }) {
            t.update_length(res.0);
            if let Some(next) = res.1 {
                t.offset += next.0;
                t.input = next.1;
                Ok(t)
            } else {
                Err(())
            }
        } else {
            let t2 = Tracker {
                offset: 0,
                input: t.input,
                children: &Cell::new(Vec::new()),
                length: &Cell::new(0),
            };
            let next = if let Ok(t3) = f.parse(t2.clone()) {
                Some((t3.offset, t3.input))
            } else {
                None
            };
            let rule = Rule {
                length: t2.length.get(),
                kind,
                children: t2.children.replace(Vec::new()),
                next: next.clone(),
            };
            t.input = t2.input;
            t.input.update(kind, rule);
            t.update_length(t2.length.get());
            if let Some((offset, pos)) = next {
                t.offset += offset;
                t.input = pos;
                Ok(t)
            } else {
                Err(())
            }
        }
    }
}

pub fn separated(symbol: impl Parser, next: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |t| many1(&pair(symbol, next))(next.parse(t)?)
}

pub fn many1(f: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |t| pair(f, &many0(f))(t)
}

pub fn many0(f: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |mut t| {
        while let Ok(new_t) = f.parse(t.clone()) {
            t = new_t
        }
        Ok(t)
    }
}

pub fn pair(f: impl Parser, g: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |t| g.parse(f.parse(t)?)
}

pub fn alt(f: impl Parser, g: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |t| {
        if let Ok(t) = f.parse(t.clone()) {
            Ok(t)
        } else {
            g.parse(t)
        }
    }
}

pub fn opt(f: impl Parser) -> impl Fn(Tracker) -> IResult {
    move |t| {
        if let Ok(t) = f.parse(t.clone()) {
            Ok(t)
        } else {
            Ok(t)
        }
    }
}
