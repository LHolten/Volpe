use std::{mem::take, rc::Rc};

use crate::{
    internal::UpgradeInternal,
    lexeme_kind::LexemeKind,
    parser::FileP,
    syntax::{Lexeme, Syntax},
    tracker::{TFunc, TInput, Tracker},
};
use crate::{logos::Logos, offset::Offset};

pub trait Packrat {
    fn parse(&mut self, string: &str, offset: Offset, length: Offset);
}

impl Packrat for Vec<Syntax> {
    fn parse(&mut self, string: &str, offset: Offset, length: Offset) {
        let mut safe = Vec::new();
        save_subtrees(self, &mut safe, offset, length);
        let (prev, offset) = prev_lexeme(self, offset);
        let is_first = prev.is_none();
        let prev = prev.unwrap_or_default();
        patch_lexeme(prev.clone(), &mut safe, offset, length, string);

        let first = if is_first {
            prev.next.upgrade().unwrap()
        } else {
            safe[0].first_lexeme() // not good
        };
        take(self);
        let input = TInput {
            lexeme: first,
            offset: Offset::default(),
            tracker: Tracker::default(),
        };
        dbg!(&input.lexeme);
        let result = FileP::parse(input);
        let tracker = Tracker::from(result);
        dbg!(&tracker.children);
        drop(safe);
        dbg!(&tracker.children);
        *self = tracker.children
    }
}

fn save_subtrees(
    children: &Vec<Syntax>,
    safe: &mut Vec<Syntax>,
    mut offset: Offset,
    mut length: Offset,
) {
    let mut child_iter = children.iter();
    while let Some(child) = child_iter.next() {
        let (mut len, reach) = child.len();
        if reach >= offset {
            if let Syntax::Rule(rule) = child {
                save_subtrees(&rule.children, safe, offset, length);
            }
        } else {
            safe.push(child.clone());
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
}

pub fn prev_lexeme(children: &Vec<Syntax>, mut offset: Offset) -> (Option<Rc<Lexeme>>, Offset) {
    for child in children {
        let len = child.len().0;
        let next_len = child.next_lexeme().length;
        if len != Offset::default() && len + next_len >= offset {
            return match child {
                Syntax::Lexeme(lexeme) => {
                    if lexeme.length >= offset {
                        (None, offset)
                    } else {
                        (Some(lexeme.clone()), offset - lexeme.length)
                    }
                }
                Syntax::Rule(rule) => prev_lexeme(&rule.children, offset),
            };
        }
        offset -= len;
    }
    unreachable!()
}

fn patch_lexeme(
    mut prev: Rc<Lexeme>,
    safe: &mut Vec<Syntax>,
    mut offset: Offset,
    length: Offset,
    string: &str,
) {
    let first = prev.next.upgrade().unwrap();
    let mut length = offset + length;

    let mut input = String::default();
    if offset < Offset::line() {
        input.push_str(&first.string[..offset.char]);
    } else {
        for (i, c) in first.string.char_indices() {
            if c == '\n' {
                offset -= Offset::line();
                if offset < Offset::line() {
                    input.push_str(&first.string[..1 + i + offset.char]);
                    break;
                }
            }
        }
    }

    input.push_str(string);

    let mut last = first;
    loop {
        let len = last.length;
        let next = last.next.upgrade();
        if len > length || next.is_none() {
            assert!(len >= length);
            break;
        }
        length -= len;
        last = next.unwrap();
    }

    if length < Offset::line() {
        input.push_str(&last.string[length.char..])
    } else {
        for (i, c) in last.string.char_indices() {
            if c == '\n' {
                length -= Offset::line();
                if length < Offset::line() {
                    input.push_str(&last.string[1 + i + length.char..]);
                    break;
                }
            }
        }
    }

    let next = last.next.take();
    let mut buffer = String::default();
    let mut buffer_length = Offset::default();
    let mut lex = LexemeKind::lexer(&input);

    while let Some(val) = lex.next() {
        buffer.push_str(lex.slice());
        buffer_length += if lex.slice() == "\n" {
            Offset::line()
        } else {
            Offset::char(lex.slice().len())
        };
        if val != LexemeKind::Error {
            let temp = Rc::new(Lexeme {
                string: take(&mut buffer),
                length: take(&mut buffer_length),
                kind: val,
                ..Lexeme::default()
            });
            prev.next.set(Rc::downgrade(&temp));
            safe.push(Syntax::Lexeme(temp.clone()));
            prev = temp
        }
    }
    if !buffer.is_empty() || next.upgrade().is_none() {
        let temp = Rc::new(Lexeme {
            string: buffer,
            ..Lexeme::default()
        });
        prev.next.set(Rc::downgrade(&temp));
        safe.push(Syntax::Lexeme(temp));
    } else {
        prev.next.set(next);
    }
}
