use logos::Logos;

use crate::{
    file::File,
    grammar::RuleKind,
    lexeme_kind::LexemeKind,
    offset::Offset,
    syntax::{Contained, Lexeme, Semicolon},
};

pub struct Yard<'a> {
    terminals: Vec<Vec<Contained<'a, ()>>>,
    stack: Vec<Lexeme<'a>>, // holds the operators that still need to be executed
    last_kind: RuleKind,
}

impl<'a> Yard<'a> {
    fn stack_is_semi(&self) -> bool {
        if let Some(op) = self.stack.last() {
            matches!(op.kind, LexemeKind::Semicolon)
        } else {
            false
        }
    }

    fn new() -> Self {
        Self {
            terminals: vec![vec![]],
            stack: vec![],
            last_kind: RuleKind::Semicolon,
        }
    }

    // this pops of all semicolon operators and returns the result
    fn reduce(&mut self) -> Semicolon<'a, ()> {
        let mut result = Semicolon::Syntax(self.terminals.pop().unwrap());
        while self.stack_is_semi() {
            result = Semicolon::Semi {
                left: self.terminals.pop().unwrap(),
                semi: self.stack.pop().unwrap(),
                right: result.into(),
            };
        }
        result
    }

    fn add_terminal(&mut self, terminal: Contained<'a, ()>) {
        // find where the terminal needs to be inserted
        // if it is on a newline then it is inserted at the start
        // if it is on the same line it is inserted just after the last item of that line
        let list = self.terminals.last_mut().unwrap();
        let prev_index = list.iter().rposition(|item| item.end() == terminal.start());
        let index = prev_index.map(|i| i + 1).unwrap_or(0);
        list.insert(index, terminal);
    }

    fn shunt(&mut self, lexeme: Lexeme<'a>) {
        let rule_kind = lexeme.kind.rule_kind();
        match rule_kind {
            RuleKind::OpeningBracket => {
                self.terminals.push(vec![]); // start a new list of terminal
                self.stack.push(lexeme)
            }
            RuleKind::ClosingBracket => {
                let inner = self.reduce();
                let open = self.stack.pop().ok_or(());
                self.add_terminal(Contained::Brackets {
                    inner: inner.into(),
                    brackets: [open, Ok(lexeme)],
                });
            }
            RuleKind::Terminal => self.add_terminal(Contained::Terminal(lexeme)),
            RuleKind::Semicolon => {
                self.terminals.push(vec![]); // start a new list of terminal
                self.stack.push(lexeme)
            }
        }
        self.last_kind = rule_kind
    }
}

impl File {
    pub fn rule(&self) -> Semicolon<()> {
        let mut yard = Yard::new();

        for (line_num, line) in self.lines.iter().enumerate() {
            let mut lexemes = LexemeKind::lexer(line);
            while let Some(kind) = lexemes.next() {
                if kind == LexemeKind::Error {
                    continue;
                }

                let span = lexemes.span();

                let lexeme = Lexeme {
                    start: Offset::new(line_num, span.start),
                    end: Offset::new(line_num, span.end),
                    kind,
                    text: &line[span.start..span.end],
                };

                yard.shunt(lexeme);
            }
        }

        let mut result = yard.reduce();
        while let Some(lexeme) = yard.stack.pop() {
            yard.add_terminal(Contained::Brackets {
                inner: result.into(),
                brackets: [Ok(lexeme), Err(())],
            });
            result = yard.reduce();
        }

        assert_eq!(yard.terminals.len(), 0);
        result
    }
}
