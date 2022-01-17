use logos::Logos;

use crate::{
    file::File,
    grammar::RuleKind,
    lexeme_kind::LexemeKind,
    offset::{Offset, Range},
    syntax::{Contained, Lexeme, Semicolon},
};

pub struct Yard<'a> {
    terminals: Vec<Vec<Contained<'a, ()>>>,
    stack: Vec<Lexeme<'a>>, // holds the operators that still need to be executed
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
        }
    }

    // this pops of all semicolon operators and returns the result
    fn reduce(&mut self) -> Vec<Vec<Contained<'a, ()>>> {
        let mut result = vec![self.terminals.pop().unwrap()];
        while self.stack_is_semi() {
            result.push(self.terminals.pop().unwrap());
            self.stack.pop().unwrap();
        }
        result
    }

    fn add_terminal(&mut self, terminal: Contained<'a, ()>) {
        let list = self.terminals.last_mut().unwrap();
        list.push(terminal);
    }

    fn shunt(&mut self, lexeme: Lexeme<'a>) {
        match lexeme.kind.rule_kind() {
            RuleKind::OpeningBracket => {
                self.terminals.push(vec![]); // start a new list of terminal
                self.stack.push(lexeme)
            }
            RuleKind::ClosingBracket => {
                let inner = self.reduce();
                let open = self.stack.pop().ok_or_else(|| self.terminals.push(vec![]));
                self.add_terminal(Contained::Brackets {
                    inner,
                    brackets: [open, Ok(lexeme)],
                });
            }
            RuleKind::Terminal => self.add_terminal(Contained::Terminal(lexeme)),
            RuleKind::Semicolon => {
                self.terminals.push(vec![]); // start a new list of terminal
                self.stack.push(lexeme)
            }
        }
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
                    kind,
                    range: Range {
                        start: Offset::new(line_num, span.start),
                        end: Offset::new(line_num, span.end),
                        text: &line[span.start..span.end],
                    },
                };

                yard.shunt(lexeme);
            }
            yard.shunt(Lexeme {
                kind: LexemeKind::Semicolon,
                range: Range {
                    start: Offset::new(line_num, line.chars().count()),
                    end: Offset::new(line_num + 1, 0),
                    text: "\n",
                },
            })
        }

        let mut result = yard.reduce();
        while let Some(lexeme) = yard.stack.pop() {
            yard.add_terminal(Contained::Brackets {
                inner: result,
                brackets: [Ok(lexeme), Err(())],
            });
            result = yard.reduce();
        }

        assert_eq!(yard.terminals.len(), 0);
        result
    }
}
