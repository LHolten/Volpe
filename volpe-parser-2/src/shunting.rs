use logos::Logos;

use crate::{file::File, lexeme_kind::LexemeKind};

#[derive(Debug)]
pub struct Rule<'a> {
    pub kind: LexemeKind,
    pub children: Vec<Rule<'a>>,
    pub slice: &'a str,
}

pub struct Yard<'a> {
    terminals: Vec<Rule<'a>>,
    operators: Vec<Rule<'a>>,
    expected: usize,
}

impl<'a> Yard<'a> {
    fn new() -> Self {
        Self {
            terminals: Vec::new(),
            operators: Vec::new(),
            expected: 1,
        }
    }

    fn shunt(&mut self, rule: Rule<'a>) {
        while self.operators.last().is_some()
            && !self.operators.last().unwrap().kind.reduce(&rule.kind)
        {
            let mut last = self.operators.pop().unwrap();
            if self.terminals.len() < self.expected {
                self.terminals.push(Rule {
                    kind: LexemeKind::Error,
                    children: Vec::new(),
                    slice: "",
                })
            }
            self.expected -= last.kind.child_count();
            last.children = self
                .terminals
                .split_off(self.terminals.len() - last.kind.child_count());
            self.expected += 1;
            self.terminals.push(last);
        }

        if self.terminals.len() > self.expected + rule.kind.child_count() - 1 {
            self.shunt(Rule {
                kind: LexemeKind::App,
                children: Vec::new(),
                slice: "",
            })
        }

        self.expected += rule.kind.child_count();
        self.expected -= 1;
        self.operators.push(rule);
    }
}

impl File {
    pub fn rule(&self) -> Rule {
        let mut yard = Yard::new();

        for line in &self.lines {
            let mut lexemes = LexemeKind::lexer(line);
            while let Some(kind) = lexemes.next() {
                if kind == LexemeKind::Error {
                    continue;
                }

                yard.shunt(Rule {
                    kind,
                    children: Vec::new(),
                    slice: lexemes.slice(),
                });

                if yard.terminals.len() + 1 < yard.expected {
                    yard.terminals.push(Rule {
                        kind: LexemeKind::Error,
                        children: Vec::new(),
                        slice: "",
                    })
                }
            }
        }

        yard.shunt(Rule {
            kind: LexemeKind::RBrace,
            children: Vec::new(),
            slice: "",
        });

        dbg!(&yard.terminals);

        yard.terminals.pop().unwrap()
    }
}
