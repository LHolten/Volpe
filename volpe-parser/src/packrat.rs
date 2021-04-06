use std::fmt;
use std::mem::take;

use crate::{
    grammar::FileP,
    lexeme_kind::LexemeKind,
    syntax::{Lexeme, OrRemaining},
    tracker::{TError, TFunc, TInput},
};
use crate::{logos::Logos, offset::Offset};

#[derive(Default)]
pub struct Parser(pub Box<Lexeme>);

impl fmt::Display for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Parser {
    // you can only use offsets that are within the text
    pub fn parse(&mut self, string: &str, offset: Offset, length: Offset) {
        let mut remaining = Vec::new();
        let (mut lexeme, mut offset) = fix_first(&mut remaining, &mut self.0, offset);
        let first = take(lexeme);
        let length = offset + length;

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

        let (mut last, mut length) = fix_last(&mut remaining, first, length);

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

        // when input is empty this will produce an unnecessary Start Lexeme
        let mut lex = LexemeKind::lexer(&input);
        while let Some(mut kind) = lex.next() {
            if lex.remainder().is_empty()
                && kind != LexemeKind::Error
                && last.next.or_remaining(&mut remaining).is_some()
            {
                let next = last.next.as_mut().unwrap();
                // Create a new lexer with more input to see if it is the same
                let mut new_input = lex.slice().to_string();
                new_input.push_str(&next.string);
                let mut new_lex = LexemeKind::lexer(&new_input);
                new_lex.next().unwrap();

                if new_lex.slice().len() != lex.slice().len() {
                    // The new lexer consumed a bigger token, time to replace the old lexer
                    input = new_input;
                    lex = LexemeKind::lexer(&input);
                    kind = lex.next().unwrap();

                    last = take(next);
                    for rule in &mut last.rules {
                        let rule = take(rule);
                        if let Some(next) = rule.next {
                            remaining.push(next);
                        }
                    }
                }
            }

            let new_length = if lex.slice() == "\n" {
                Offset::line()
            } else {
                Offset::char(lex.slice().len() as u32)
            };

            if kind != LexemeKind::Error {
                if lexeme.length != Offset::default() {
                    lexeme.next = Some(Default::default());
                    lexeme = lexeme.next.as_mut().unwrap();
                }
                *lexeme = Box::new(Lexeme {
                    string: lex.slice().to_string(),
                    token_length: new_length,
                    length: new_length,
                    kind,
                    ..Lexeme::default()
                });
            } else {
                lexeme.string.push_str(lex.slice());
                lexeme.length += new_length;
            }
        }
        lexeme.next = last.next;

        let mut lexeme_option = Some(take(&mut self.0));
        FileP::parse(TInput {
            lexeme: &mut lexeme_option,
            length: Offset::default(),
            error: TError {
                remaining,
                sensitive_length: Offset::default(),
            },
        })
        .ok()
        .unwrap();
        self.0 = lexeme_option.unwrap();
    }
}

fn fix_first<'a>(
    remaining: &mut Vec<Box<Lexeme>>,
    lexeme: &'a mut Box<Lexeme>,
    offset: Offset,
) -> (&'a mut Box<Lexeme>, Offset) {
    let ptr = lexeme as *mut _;
    let mut furthest = (lexeme.length, &mut lexeme.next);
    for rule in &mut lexeme.rules {
        if rule.sensitive_length >= offset {
            let rule = take(rule);
            if let Some(next) = rule.next {
                remaining.push(next);
            }
        } else if rule.length > furthest.0 || rule.length == furthest.0 && furthest.1.is_none() {
            furthest = (rule.length, &mut rule.next);
        }
    }
    if lexeme.length >= offset {
        return (unsafe { &mut *(ptr) }, offset);
    }
    let next = furthest.1.or_remaining(remaining).as_mut().unwrap();
    fix_first(remaining, next, offset - furthest.0)
}

fn fix_last(
    remaining: &mut Vec<Box<Lexeme>>,
    mut lexeme: Box<Lexeme>,
    length: Offset,
) -> (Box<Lexeme>, Offset) {
    let mut furthest = (lexeme.length, &mut lexeme.next);
    for rule in &mut lexeme.rules {
        if rule.length >= length {
            let rule = take(rule);
            if let Some(next) = rule.next {
                remaining.push(next);
            }
        } else if rule.length > furthest.0 || rule.length == furthest.0 && furthest.1.is_none() {
            furthest = (rule.length, &mut rule.next);
        }
    }
    if lexeme.length >= length {
        return (lexeme, length);
    }
    let next = furthest.1.or_remaining(remaining).as_mut().unwrap();
    fix_last(remaining, take(next), length - furthest.0)
}

impl<'a> Parser {
    pub fn lexeme_at_offset(&'a self, target: Offset) -> &'a Lexeme {
        let mut current = Offset::default();
        let mut lexeme = self.0.as_ref();

        'next: loop {
            for rule in &lexeme.rules {
                if rule.length == Offset::default() {
                    continue;
                }
                // Rules are organised from largest to smallest.
                // We look for the first rule which is smaller than the target.
                if current + rule.length <= target {
                    if let Some(next) = &rule.next {
                        current += rule.length;
                        lexeme = next.as_ref();
                        continue 'next;
                    }
                }
            }

            if current + lexeme.length > target {
                break;
            }

            if let Some(next) = &lexeme.next {
                current += lexeme.length;
                lexeme = next.as_ref();
                continue 'next;
            }

            // If we got here it means that we reached the end of a lexeme chain.
            // This should never happen because that would mean that the lexeme
            // we are looking for was not in this rule.
            // In that case we would have skipped this chain earlier on.
            unreachable!()
        }

        lexeme
    }
}
