use std::{cell::Cell, fmt::Debug, mem::take, rc::Rc};

use crate::lexer::Lexem;
use crate::logos::Logos;

pub type IResult<'t> = Result<Tracker<'t>, ()>;

#[derive(Clone)]
struct Rule {
    length: usize,
    children: Vec<RuleRef>,
    next: Option<(usize, SharedPosition)>,
}

#[derive(Clone)]
pub struct RuleRef(pub Option<RuleKind>, pub SharedPosition);

impl RuleRef {
    fn is_succes(&self) -> bool {
        self.0.is_none()
            || self
                .1
                .with_rule(self.0.unwrap(), |r| {
                    r.next.is_some() && r.next.as_ref().unwrap().0 != 0
                })
                .unwrap()
    }
}

impl Debug for RuleRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(kind) = self.0 {
            f.write_str(&format!("{:?}", kind))?;
            self.1
                .clone_rule(kind)
                .unwrap()
                .children
                .into_iter()
                .filter(RuleRef::is_succes)
                .collect::<Vec<RuleRef>>()
                .fmt(f)
        } else {
            self.1.with_pos(|pos| pos.lexem.fmt(f))
        }
    }
}

#[derive(Default, Clone)]
pub struct SharedPosition(Rc<Cell<Position>>);

#[derive(Default)]
struct Position {
    lexem: String, // white space and unknown in front
    kind: Lexem,
    rules: [Option<Rule>; 9],
    next: Option<SharedPosition>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RuleKind {
    Expr,
    Stmt,
    App,
    Func,
    Or,
    And,
    Op1,
    Op2,
    Op3,
}

impl SharedPosition {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(Position {
            lexem: " ".to_string(),
            ..Default::default()
        })))
    }

    fn with_pos<R>(&self, f: impl FnOnce(&mut Position) -> R) -> R {
        let mut pos = self.0.replace(Position::default());
        let res = f(&mut pos);
        self.0.set(pos);
        res
    }

    fn with_rule<R>(&self, kind: RuleKind, f: impl FnOnce(&Rule) -> R) -> Option<R> {
        self.with_pos(|pos| pos.rules[kind as usize].as_ref().map(f))
    }

    fn update(&self, kind: RuleKind, rule: Rule) {
        self.with_pos(|pos| pos.rules[kind as usize] = Some(rule))
    }

    fn len(&self) -> usize {
        self.with_pos(|pos| pos.lexem.len())
    }

    fn next(&self) -> Self {
        self.with_pos(|pos| pos.next.as_ref().unwrap().clone())
    }

    fn kind(&self) -> Lexem {
        self.with_pos(|pos| pos.kind)
    }

    fn clone_rule(&self, kind: RuleKind) -> Option<Rule> {
        self.with_pos(|pos| pos.rules[kind as usize].clone())
    }

    pub fn patch(
        &self,
        kind: Option<RuleKind>,
        string: &str,
        mut offset: usize,
        length: usize,
    ) -> Result<usize, ()> {
        if let Some(k) = kind {
            let len = self.with_rule(k, |r| r.length).unwrap();
            if len > offset {
                let rule = self.with_pos(|pos| pos.rules[k as usize].take().unwrap());

                for child in rule.children {
                    offset = child.1.patch(child.0, string, offset, length)?;
                }
            }
            offset -= self.with_rule(k, |r| (r.next.as_ref().unwrap().0)).unwrap()
        } else {
            let len = self.len();
            if len >= offset {
                let first_overlap = len - offset;
                let mut last = self.clone();
                let mut last_offset = 0;
                loop {
                    last_offset += last.len();
                    if last_offset > offset + length {
                        break;
                    }
                    last = last.next();
                }
                let last_overlap = last_offset - offset - length;

                let mut input = String::default();
                self.with_pos(|pos| input.push_str(&pos.lexem[..pos.lexem.len() - first_overlap]));
                input.push_str(string);
                last.with_pos(|pos| input.push_str(&pos.lexem[last_overlap..]));

                let mut buffer = String::default();
                let mut lex = Lexem::lexer(&input);
                let mut current = self.clone();
                while let Some(val) = lex.next() {
                    buffer.push_str(lex.slice());
                    if val != Lexem::Error {
                        let temp = SharedPosition::default();
                        current.0.set(Position {
                            lexem: take(&mut buffer),
                            kind: val,
                            next: Some(temp.clone()),
                            rules: Default::default(),
                        });
                        current = temp;
                    }
                }
                current.with_pos(|pos| pos.next = Some(last.next()));
                return Err(());
            }
            offset -= len;
        }
        Ok(offset)
    }

    pub fn parse(&self, func: impl Fn(Tracker) -> IResult) -> Result<(), ()> {
        func(Tracker {
            pos: self.clone(),
            offset: 0,
            children: &Default::default(),
            length: &Default::default(),
        })?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Tracker<'t> {
    pos: SharedPosition,
    offset: usize,
    children: &'t Cell<Vec<RuleRef>>,
    length: &'t Cell<usize>,
}

impl<'t> Tracker<'t> {
    pub fn add_child(&self, kind: Option<RuleKind>, pos: SharedPosition) {
        let mut children = self.children.replace(Vec::new());
        children.push(RuleRef(kind, pos));
        self.children.set(children)
    }

    pub fn update_length(&self, length: usize) {
        self.length
            .set((self.offset + length).max(self.length.get()));
    }
}

pub fn tag(kind: impl Into<usize> + Copy) -> impl Fn(Tracker) -> IResult {
    move |mut t: Tracker| {
        t.add_child(None, t.pos.clone());
        t.update_length(t.pos.len());
        if 1 << t.pos.kind() as usize & kind.into() != 0 {
            t.offset += t.pos.len();
            t.pos = t.pos.next();
            Ok(t)
        } else {
            Err(())
        }
    }
}

pub fn rule(kind: RuleKind, f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |mut t: Tracker| {
        t.add_child(Some(kind), t.pos.clone());
        if let Some(res) = t.pos.with_rule(kind, |r| {
            if let Some(next) = r.next.clone() {
                (r.length, Some(next))
            } else {
                (r.length, None)
            }
        }) {
            t.update_length(res.0);
            if let Some(next) = res.1 {
                t.offset += next.0;
                t.pos = next.1;
                Ok(t)
            } else {
                Err(())
            }
        } else {
            let t2 = Tracker {
                offset: 0,
                pos: t.pos,
                children: &Cell::new(Vec::new()),
                length: &Cell::new(0),
            };
            let next = if let Ok(t3) = f(t2.clone()) {
                Some((t3.offset, t3.pos))
            } else {
                None
            };
            let rule = Rule {
                length: t2.length.get(),
                children: t2.children.replace(Vec::new()),
                next: next.clone(),
            };
            t.pos = t2.pos;
            t.pos.update(kind, rule);
            t.update_length(t2.length.get());
            if let Some((offset, pos)) = next {
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
