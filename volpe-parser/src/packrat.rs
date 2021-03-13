use std::{cell::Cell, fmt::Debug, mem::take, rc::Rc};

use crate::{
    lexem_kind::LexemKind,
    parser::expr,
    syntax::{Lexem, Rule, Syntax},
    tracker::Tracker,
    with_internal::WithInternal,
};
use crate::{logos::Logos, offset::Offset};

pub struct Packrat {
    syntax: Option<Syntax>,
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

    fn len(&self) -> (Offset, Offset) {
        match self {
            Syntax::Lexem(pos, success) => pos.with(|p| {
                (
                    if *success {
                        p.length
                    } else {
                        Offset::default()
                    },
                    p.length,
                )
            }),
            Syntax::Rule(rule) => rule.with(|r| {
                (
                    r.success.as_ref().map_or(Offset::default(), |v| v.0),
                    r.length,
                )
            }),
        }
    }

    fn get_pos(&self) -> Rc<Cell<Lexem>> {
        match self {
            Syntax::Lexem(pos, _) => pos.clone(),
            Syntax::Rule(rule) => rule.with(|r| r.children[0].get_pos()),
        }
    }

    fn patch_rule(
        &self,
        safe: &mut Vec<Syntax>,
        mut offset: Offset,
        mut length: Offset,
    ) -> Option<(Rc<Cell<Lexem>>, Offset, Offset)> {
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
                        offset = Offset::default();
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
        first: Rc<Cell<Lexem>>,
        mut offset: Offset,
        mut length: Offset,
        string: &str,
        safe: &mut Vec<Syntax>,
    ) {
        length = offset + length;

        let mut input = String::default();
        let first_str = first.with(|p| p.lexem.clone());
        if offset < Offset::line() {
            input.push_str(&first_str[..offset.char]);
        } else {
            for (i, c) in first_str.char_indices() {
                if c == '\n' {
                    offset -= Offset::line();
                    if offset < Offset::line() {
                        input.push_str(&first_str[..1 + i + offset.char]);
                        break;
                    }
                }
            }
        }

        input.push_str(string);

        let mut last = first.clone();
        loop {
            let len = last.with(|p| p.length);
            let next = last.with(|p| p.next.upgrade());
            if len > length || next.is_none() {
                assert!(len >= length);
                break;
            }
            length -= len;
            last = next.unwrap();
        }

        let last_str = last.with(|p| p.lexem.clone());
        if length < Offset::line() {
            input.push_str(&last_str[length.char..])
        } else {
            for (i, c) in last_str.char_indices() {
                if c == '\n' {
                    length -= Offset::line();
                    if length < Offset::line() {
                        input.push_str(&last_str[1 + i + length.char..]);
                        break;
                    }
                }
            }
        }

        let last_internal = last.replace(Lexem::default()); //keep this from being overwritten

        let mut buffer = String::default();
        let mut buffer_length = Offset::default();
        let mut lex = LexemKind::lexer(&input);
        safe.push(Syntax::Lexem(first.clone(), false));
        let mut current = first;
        while let Some(val) = lex.next() {
            buffer.push_str(lex.slice());
            buffer_length += if lex.slice() == "\n" {
                Offset::line()
            } else {
                Offset::char(lex.slice().len())
            };
            if val != LexemKind::Error {
                let mut temp = Default::default();
                if lex.remainder().is_empty() {
                    if let Some(next) = last_internal.next.upgrade() {
                        temp = next
                    }
                };
                current.set(Lexem {
                    lexem: take(&mut buffer),
                    length: take(&mut buffer_length),
                    kind: val,
                    next: Rc::downgrade(&temp),
                    rules: Default::default(),
                });
                safe.push(Syntax::Lexem(temp.clone(), false));
                current = temp;
            }
        }

        if !buffer.is_empty() {
            current.set(Lexem {
                lexem: buffer,
                ..Lexem::default()
            });
        }
    }

    pub fn parse(self, string: &str, offset: Offset, length: Offset) -> Syntax {
        let pos = self.get_pos();
        let mut safe = Vec::new();
        let res = self.patch_rule(&mut safe, offset, length).unwrap();
        Self::patch_lexem(res.0, res.1, res.2, string, &mut safe);
        let tracker = Tracker {
            pos,
            offset: Offset::default(),
            children: &Default::default(),
            length: &Default::default(),
        };
        drop(self);
        expr(tracker.clone()).unwrap();
        drop(safe);
        Syntax::Rule(Rc::new(Cell::new(Rule {
            length: tracker.length.get(),
            children: tracker.children.take(),
            success: None,
        })))
    }

    // pub fn text(&self) -> String {
    //     self.get_pos().with(|p| format!("{:?}", p))
    // }
}
