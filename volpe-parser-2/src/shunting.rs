use logos::Logos;

use crate::{file::File, lexeme_kind::LexemeKind};

pub struct Rule<'a> {
    pub kind: LexemeKind,
    pub children: Vec<Rule<'a>>,
    pub slice: &'a str,
}

impl File {
    pub fn rule(&self) -> Rule {
        let mut terminals = Vec::new();
        let mut operators: Vec<Rule> = Vec::new();

        for line in &self.lines {
            let mut lexemes = LexemeKind::lexer(line);
            while let Some(kind) = lexemes.next() {
                if kind == LexemeKind::Error {
                    continue;
                }
                while !operators.last().unwrap().kind.reduce(&kind) {
                    let mut last = operators.pop().unwrap();
                    last.children = terminals.split_off(terminals.len() - last.kind.child_count());
                    terminals.push(last);
                }
                operators.push(Rule {
                    kind,
                    children: Vec::new(),
                    slice: lexemes.slice(),
                })
            }
        }

        while let Some(mut last) = operators.pop() {
            last.children = terminals.split_off(terminals.len() - last.kind.child_count());
            terminals.push(last);
        }

        terminals.pop().unwrap()
    }
}
