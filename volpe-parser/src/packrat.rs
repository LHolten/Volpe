use std::mem::take;

use crate::{
    grammar::FileP,
    lexeme_kind::LexemeKind,
    syntax::Lexeme,
    tracker::{TError, TFunc, TInput},
};
use crate::{logos::Logos, offset::Offset};

pub struct Parser(Option<Box<Lexeme>>);

impl Default for Parser {
    fn default() -> Self {
        Self(Some(Default::default()))
    }
}

impl Parser {
    // you can only use offsets that are within the text
    pub fn parse(&mut self, string: &str, offset: Offset, length: Offset) {
        let mut remaining = Vec::new();
        let (mut next, mut offset) = fix_first(&mut remaining, &mut self.0, offset);
        let first = take(next).unwrap();

        let mut input = String::default();
        if offset < Offset::line() {
            input.push_str(&first.string[..offset.char as usize]);
        } else {
            for (i, c) in first.string.char_indices() {
                if c == '\n' {
                    offset -= Offset::line();
                    if offset < Offset::line() {
                        input.push_str(&first.string[..1 + i + offset.char as usize]);
                        break;
                    }
                }
            }
        }

        input.push_str(string);

        let (last, mut length) = fix_last(&mut remaining, first, offset + length);

        if length < Offset::line() {
            input.push_str(&last.string[length.char as usize..])
        } else {
            for (i, c) in last.string.char_indices() {
                if c == '\n' {
                    length -= Offset::line();
                    if length < Offset::line() {
                        input.push_str(&last.string[1 + i + length.char as usize..]);
                        break;
                    }
                }
            }
        }

        let mut buffer = String::default();
        let mut buffer_length = Offset::default();
        let mut lex = LexemeKind::lexer(&input);

        while let Some(val) = lex.next() {
            buffer.push_str(lex.slice());
            buffer_length += if lex.slice() == "\n" {
                Offset::line()
            } else {
                Offset::char(lex.slice().len() as u32)
            };
            if val != LexemeKind::Error {
                *next = Some(Box::new(Lexeme {
                    string: take(&mut buffer),
                    length: take(&mut buffer_length),
                    kind: val,
                    ..Lexeme::default()
                }));
                next = &mut next.as_mut().unwrap().next;
            }
        }

        if !buffer.is_empty() || last.length == Offset::default() {
            *next = Some(Box::new(Lexeme {
                string: take(&mut buffer),
                length: take(&mut buffer_length),
                ..Lexeme::default()
            }))
        } else {
            *next = last.next;
        }

        FileP::parse(TInput {
            lexeme: &mut self.0,
            length: Offset::default(),
            error: TError {
                remaining,
                sensitive_length: Offset::default(),
            },
        })
        .ok()
        .unwrap();
    }
}

pub fn fix_first<'a>(
    remaining: &mut Vec<Box<Lexeme>>,
    lexeme: &'a mut Option<Box<Lexeme>>,
    offset: Offset,
) -> (&'a mut Option<Box<Lexeme>>, Offset) {
    if lexeme.as_mut().unwrap().length >= offset {
        return (lexeme, offset);
    }
    let lexeme = lexeme.as_mut().unwrap();
    let mut furthest = (lexeme.length, &mut lexeme.next);
    for rule in &mut lexeme.rules {
        if rule.sensitive_length >= offset {
            let rule = take(rule);
            if let Some(next) = rule.next {
                remaining.push(next);
            }
        } else if rule.length > furthest.0 {
            furthest = (rule.length, &mut rule.next);
        }
    }
    if furthest.1.is_none() {
        *furthest.1 = Some(remaining.pop().unwrap());
    }
    fix_first(remaining, furthest.1, offset - furthest.0)
}

pub fn fix_last(
    remaining: &mut Vec<Box<Lexeme>>,
    mut lexeme: Box<Lexeme>,
    length: Offset,
) -> (Box<Lexeme>, Offset) {
    if lexeme.length > length || lexeme.length == Offset::default() {
        return (lexeme, length);
    }
    let mut furthest = (lexeme.length, &mut lexeme.next);
    for rule in &mut lexeme.rules {
        if rule.length >= length {
            let rule = take(rule);
            if let Some(next) = rule.next {
                remaining.push(next);
            }
        } else if rule.length > furthest.0 {
            furthest = (rule.length, &mut rule.next);
        }
    }
    if furthest.1.is_none() {
        *furthest.1 = Some(remaining.pop().unwrap());
    }
    fix_last(
        remaining,
        take(&mut lexeme.next).unwrap(),
        length - lexeme.length,
    )
}
