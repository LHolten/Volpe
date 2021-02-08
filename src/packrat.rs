use std::{cell::Cell, mem::take, rc::Rc};

use crate::lexer::Lexem;
use crate::logos::Logos;

pub type IResult<'t> = Result<Tracker<'t>, ()>;

pub struct Rule {
    length: usize,
    kind: RuleKind,
    children: Vec<(Option<RuleKind>, SharedPosition)>,
    next: Option<(usize, SharedPosition)>,
}

#[derive(Default, Clone)]
struct SharedPosition(Rc<Cell<Position>>);

#[derive(Default)]
pub struct Position {
    lexem: String, // white space and unknown in front
    kind: Lexem,
    rules: [Option<Rule>; 10],
    next: Option<SharedPosition>,
}

#[derive(Clone, Copy, PartialEq)]
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
    Tuple,
}

impl SharedPosition {
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

    fn take_lexem(&mut self) -> String {
        self.with_pos(|pos| take(&mut pos.lexem))
    }

    pub fn patch(&self, kind: RuleKind, string: &str, offset: usize, length: usize) {
        let rule = self.with_pos(|pos| pos.rules[kind as usize].take().unwrap());

        let mut child_offset = 0;
        for child in rule.children {
            if let Some(k) = child.0 {
                let child_length = child.1.with_rule(k, |r| r.length).unwrap();
                if child_length + child_offset > offset {
                    child.1.patch(k, string, offset - child_offset, length);
                    break;
                }
                child_offset += child
                    .1
                    .with_rule(k, |r| (r.next.as_ref().unwrap().0))
                    .unwrap();
            } else {
                let child_length = child.1.len();
                if child_length + child_offset >= offset {
                    let mut first = child.1;
                    let first_overlap = child_length + child_offset - offset;
                    let mut last = first.clone();
                    loop {
                        child_offset += last.len();
                        last = last.next();
                        if child_offset + last.len() > offset + length {
                            break;
                        }
                    }
                    let last_overlap = child_offset + last.len() - offset - length;

                    let mut new_input = first.take_lexem();
                    new_input.truncate(new_input.len() - first_overlap);
                    new_input.push_str(string);
                    new_input.push_str(&last.take_lexem()[last_overlap..]);

                    let mut buffer = String::default();
                    let mut lex = Lexem::lexer(&new_input);
                    while let Some(val) = lex.next() {
                        buffer.push_str(lex.slice());
                        if val != Lexem::Error {
                            let temp = SharedPosition::default();
                            first.0.set(Position {
                                lexem: take(&mut buffer),
                                kind: val,
                                next: Some(temp.clone()),
                                rules: Default::default(),
                            });
                            first = temp;
                        }
                    }
                    first.with_pos(|pos| pos.next = Some(last.next()));
                    break;
                }
                child_offset += child_length
            }
        }
    }
}

#[derive(Clone)]
pub struct Tracker<'t> {
    pos: SharedPosition,
    offset: usize,
    children: &'t Cell<Vec<(Option<RuleKind>, SharedPosition)>>,
    length: &'t Cell<usize>,
}

impl<'t> Tracker<'t> {
    pub fn add_child(&self, kind: Option<RuleKind>, pos: SharedPosition) {
        let mut children = self.children.replace(Vec::new());
        children.push((kind, pos));
        self.children.set(children)
    }

    pub fn update_length(&self, length: usize) {
        self.length
            .set((self.offset + length).max(self.length.get()));
    }
}

pub fn tag(kind: Lexem) -> impl Fn(Tracker) -> IResult {
    move |mut t: Tracker| {
        t.add_child(None, t.pos.clone());
        t.update_length(t.pos.len());
        if t.pos.kind() == kind {
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
                kind,
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

pub fn many1(f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |t| pair(&f, many0(&f))(t)
}

pub fn many0(f: impl Fn(Tracker) -> IResult) -> impl Fn(Tracker) -> IResult {
    move |mut t| {
        while let Ok(new_t) = f(t.clone()) {
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
