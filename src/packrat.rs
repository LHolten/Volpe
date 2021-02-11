use std::{
    cell::Cell,
    fmt::Debug,
    mem::take,
    rc::{Rc, Weak},
};

use crate::logos::Logos;
use crate::{lexer::Lexem, parser::expr};

pub type IResult<'t> = Result<Tracker<'t>, ()>;

#[derive(Clone, Default)]
pub struct Rule {
    length: usize,
    children: Vec<Syntax>,
    success: Option<(usize, Rc<Cell<Position>>, RuleKind)>,
}

#[derive(Default, Clone)]
pub struct Position {
    lexem: String, // white space and unknown in front
    kind: Lexem,
    rules: [Weak<Cell<Rule>>; 9],
    next: Weak<Cell<Position>>,
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.lexem)?;
        if let Some(next) = self.next.upgrade() {
            next.with(|n| n.fmt(f))
        } else {
            Ok(())
        }
    }
}

trait WithInternal<T> {
    fn with<R>(&self, func: impl FnOnce(&mut T) -> R) -> R;
}

impl<T: Default> WithInternal<T> for Cell<T> {
    fn with<R>(&self, func: impl FnOnce(&mut T) -> R) -> R {
        let mut temp = self.replace(T::default());
        let res = func(&mut temp);
        self.set(temp);
        res
    }
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

#[derive(Clone)]
pub enum Syntax {
    Lexem(Rc<Cell<Position>>, bool),
    Rule(Rc<Cell<Rule>>),
}

impl Debug for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Syntax::Lexem(pos, _) => pos.with(|p| p.lexem.fmt(f)),
            Syntax::Rule(rule) => {
                let kind = rule.with(|r| {
                    r.success
                        .as_ref()
                        .map_or(String::new(), |v| format!("{:?}", v.2))
                });
                f.write_str(&kind)?;
                let children = rule.with(|r| r.children.clone());
                children
                    .into_iter()
                    .filter(Syntax::is_success)
                    .collect::<Vec<Syntax>>()
                    .fmt(f)
            }
        }
    }
}

impl Default for Syntax {
    fn default() -> Self {
        Self::Lexem(Default::default(), false)
    }
}

impl Syntax {
    fn is_success(&self) -> bool {
        match self {
            Syntax::Lexem(_, success) => *success,
            Syntax::Rule(rule) => rule.with(|r| r.success.is_some()),
        }
    }

    fn len(&self) -> (usize, usize) {
        match self {
            Syntax::Lexem(pos, _) => pos.with(|p| (p.lexem.len(), p.lexem.len())),
            Syntax::Rule(rule) => rule.with(|r| (r.success.as_ref().map_or(0, |v| v.0), r.length)),
        }
    }

    fn get_pos(&self) -> Rc<Cell<Position>> {
        match self {
            Syntax::Lexem(pos, _) => pos.clone(),
            Syntax::Rule(rule) => rule.with(|r| r.children[0].get_pos()),
        }
    }

    fn patch_rule(
        &self,
        safe: &mut Vec<Syntax>,
        mut offset: usize,
        mut length: usize,
    ) -> Option<(Rc<Cell<Position>>, usize, usize)> {
        match self {
            Syntax::Rule(rule) => rule.with(|rule| {
                let mut child_iter = take(&mut rule.children).into_iter();
                let mut res = None;
                while let Some(child) = child_iter.next() {
                    let (mut len, reach) = child.len();

                    if reach >= offset {
                        let new_res = child.patch_rule(safe, offset, length);
                        res = res.or(new_res);
                    } else {
                        safe.push(child);
                    }

                    if len > offset {
                        len -= offset;
                        if len > length {
                            break;
                        } else {
                            length -= len;
                        }
                        offset = 0;
                    } else {
                        offset -= len;
                    }
                }
                safe.extend(child_iter);
                res
            }),
            Syntax::Lexem(pos, _) => {
                safe.push(self.clone());
                Some((pos.clone(), offset, length))
            }
        }
    }

    fn patch_lexem(
        first: Rc<Cell<Position>>,
        offset: usize,
        length: usize,
        string: &str,
        safe: &mut Vec<Syntax>,
    ) {
        let first_len = first.with(|p| p.lexem.len());
        let first_overlap = first_len - offset;
        let mut last = first.clone();
        let mut last_len = 0;
        loop {
            last_len += last.with(|p| p.lexem.len());
            let next = last.with(|p| p.next.upgrade());
            if last_len > offset + length || next.is_none() {
                assert!(last_len >= offset + length);
                break;
            }
            last = next.unwrap();
        }
        let last_extra = last_len - offset - length;

        let mut input = String::default();
        first.with(|p| input.push_str(&p.lexem[..p.lexem.len() - first_overlap]));
        input.push_str(string);
        last.with(|p| input.push_str(&p.lexem[p.lexem.len() - last_extra..]));

        let last_internal = last.replace(Position::default()); //keep this from being overwritten

        let mut buffer = String::default();
        let mut lex = Lexem::lexer(&input);
        safe.push(Syntax::Lexem(first.clone(), false));
        let mut current = first;
        while let Some(val) = lex.next() {
            buffer.push_str(lex.slice());
            if val != Lexem::Error {
                let temp = Default::default();
                current.set(Position {
                    lexem: take(&mut buffer),
                    kind: val,
                    next: Rc::downgrade(&temp),
                    rules: Default::default(),
                });
                safe.push(Syntax::Lexem(temp.clone(), false));
                current = temp;
            }
        }

        if let Some(next) = last_internal.next.upgrade() {
            assert!(buffer.is_empty());
            current.swap(&next);
        } else {
            current.set(Position {
                lexem: buffer,
                ..Default::default()
            });
        }
    }

    pub fn parse(self, string: &str, offset: usize, length: usize) -> Syntax {
        let pos = self.get_pos();
        let mut safe = Vec::new();
        let res = self.patch_rule(&mut safe, offset, length).unwrap();
        Self::patch_lexem(res.0, res.1, res.2, string, &mut safe);
        let tracker = Tracker {
            pos,
            offset: 0,
            children: &Default::default(),
            length: &Default::default(),
        };
        drop(self);
        expr(tracker.clone()).unwrap();
        drop(safe);
        Syntax::Rule(Rc::new(Cell::new(Rule {
            length: tracker.length.get(),
            children: tracker.children.replace(Default::default()),
            success: None,
        })))
    }

    pub fn text(&self) -> String {
        self.get_pos().with(|p| format!("{:?}", p))
    }
}

#[derive(Clone)]
pub struct Tracker<'t> {
    pos: Rc<Cell<Position>>,
    offset: usize,
    children: &'t Cell<Vec<Syntax>>,
    length: &'t Cell<usize>,
}

impl<'t> Tracker<'t> {
    pub fn add_child(&self, child: Syntax) {
        self.children.with(|children| children.push(child));
    }

    pub fn update_length(&self, length: usize) {
        self.length
            .set((self.offset + length).max(self.length.get()));
    }
}

pub fn tag(kind: impl Into<usize> + Copy) -> impl Fn(Tracker) -> IResult {
    move |mut t: Tracker| {
        let length = t.pos.with(|p| p.lexem.len());
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
                offset: 0,
                pos: t.pos,
                children: &Cell::new(Vec::new()),
                length: &Cell::new(0),
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
