use logos::Logos;

use crate::{
    file::File,
    grammar::RuleKind,
    lexeme_kind::LexemeKind,
    offset::Offset,
    syntax::{Lexeme, Syntax},
};

pub struct Yard<'a> {
    terminals: Vec<Syntax<'a, ()>>,
    stack: Vec<Option<Lexeme<'a>>>, // holds the operators that still need to be executed
    last_kind: RuleKind,
}

impl<'a> Yard<'a> {
    fn stack_can_hold(&self, lexeme_kind: &LexemeKind) -> bool {
        if let Some(op) = self.stack.last() {
            if let Some(op) = op {
                op.kind.reduce(lexeme_kind)
            } else {
                LexemeKind::App.reduce(lexeme_kind)
            }
        } else {
            true
        }
    }

    fn new() -> Self {
        Self {
            terminals: Vec::new(),
            stack: Vec::new(),
            last_kind: RuleKind::Operator,
        }
    }

    fn pop_terminal(&mut self) -> Syntax<'a, ()> {
        self.terminals.pop().unwrap()
    }

    fn begin_terminal(&mut self) {
        if matches!(
            self.last_kind,
            RuleKind::Terminal | RuleKind::ClosingBracket
        ) {
            self.begin_operator(&LexemeKind::App);
            self.stack.push(None);
        }
    }

    fn begin_operator(&mut self, lexeme_kind: &LexemeKind) {
        if matches!(
            self.last_kind,
            RuleKind::Operator | RuleKind::OpeningBracket
        ) {
            self.terminals.push(Syntax::Terminal(Err(())));
        }

        while !self.stack_can_hold(lexeme_kind) {
            let second = self.pop_terminal().into(); // needed to get the correct operand order
            let first = self.pop_terminal().into();
            self.terminals.push(Syntax::Operator {
                operator: self.stack.pop().unwrap(),
                operands: [first, second],
            });
        }
    }

    fn shunt(&mut self, lexeme: Lexeme<'a>) {
        let rule_kind = lexeme.kind.rule_kind();
        match rule_kind {
            RuleKind::OpeningBracket => {
                self.begin_terminal();
                self.stack.push(Some(lexeme));
            }
            RuleKind::ClosingBracket => {
                self.begin_operator(&lexeme.kind);
                let open = self.stack.pop().map(Option::unwrap).ok_or(());
                let inner = self.pop_terminal().into();
                self.terminals.push(Syntax::Brackets {
                    inner,
                    brackets: [open, Ok(lexeme)],
                });
            }
            RuleKind::Terminal => {
                self.begin_terminal();
                self.terminals.push(Syntax::Terminal(Ok(lexeme)))
            }
            RuleKind::Operator => {
                self.begin_operator(&lexeme.kind);
                self.stack.push(Some(lexeme))
            }
        }
        self.last_kind = rule_kind
    }
}

impl File {
    pub fn rule(&self) -> Syntax<()> {
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

        while let Some(lexeme) = {
            yard.begin_operator(&LexemeKind::RRoundBracket);
            yard.stack.pop()
        } {
            let inner = yard.pop_terminal().into();
            yard.terminals.push(Syntax::Brackets {
                inner,
                brackets: [Ok(lexeme.unwrap()), Err(())],
            });
            yard.last_kind = RuleKind::ClosingBracket;
        }

        assert_eq!(yard.terminals.len(), 1);
        yard.pop_terminal()
    }
}
