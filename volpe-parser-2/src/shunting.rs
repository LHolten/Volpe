use logos::Logos;

use crate::{file::File, grammar::RuleKind, lexeme_kind::LexemeKind};

#[derive(Debug)]
pub struct Rule<'a> {
    pub slice: &'a str,
    pub children: Vec<Rule<'a>>,
    pub kind: LexemeKind,
}

pub struct Yard<'a> {
    terminals: Vec<Rule<'a>>,
    operators: Vec<Rule<'a>>,
    last_kind: RuleKind,
}

const ERROR_RULE: Rule = Rule {
    slice: "",
    children: Vec::new(),
    kind: LexemeKind::Error,
};

const APP_RULE: Rule = Rule {
    slice: "",
    children: Vec::new(),
    kind: LexemeKind::App,
};

impl<'a> Yard<'a> {
    fn operator_reduce(&self, kind: &LexemeKind) -> bool {
        self.operators
            .last()
            .map_or(&LexemeKind::Error, |r| &r.kind)
            .reduce(kind)
    }

    fn terminal_pop(&mut self) -> Rule<'a> {
        self.terminals.pop().unwrap_or(ERROR_RULE)
    }

    fn operator_pop(&mut self) -> Rule<'a> {
        self.operators.pop().unwrap_or(ERROR_RULE)
    }

    fn new() -> Self {
        Self {
            terminals: Vec::new(),
            operators: Vec::new(),
            last_kind: RuleKind::Operator,
        }
    }

    fn begin_terminal(&mut self) {
        if matches!(self.last_kind, RuleKind::Terminal | RuleKind::ClosingBrace) {
            self.shunt(APP_RULE)
        }
    }

    fn shunt_operator(&mut self, rule: Rule<'a>) {
        if matches!(self.last_kind, RuleKind::Operator | RuleKind::OpeningBrace) {
            self.terminals.push(ERROR_RULE);
        }

        while !self.operator_reduce(&rule.kind) {
            let mut last = self.operator_pop();
            let tmp = self.terminal_pop();
            last.children.push(self.terminal_pop());
            last.children.push(tmp);
            self.terminals.push(last);
        }
        self.operators.push(rule);
    }

    fn shunt(&mut self, rule: Rule<'a>) {
        let rule_kind = rule.kind.rule_kind();
        match rule_kind {
            RuleKind::OpeningBrace => {
                self.begin_terminal();
                self.operators.push(rule);
            }
            RuleKind::ClosingBrace => {
                self.shunt_operator(rule);
                let mut last = self.operator_pop();
                last.children.push(self.operator_pop());
                last.children.push(self.terminal_pop());
                self.terminals.push(last);
            }
            RuleKind::Terminal => {
                self.begin_terminal();
                self.terminals.push(rule)
            }
            RuleKind::Operator => self.shunt_operator(rule),
        }
        self.last_kind = rule_kind
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
                    slice: lexemes.slice(),
                    children: Vec::new(),
                    kind,
                });
            }
        }

        while yard.terminals.len() > 1 {
            dbg!(&yard.terminals);
            yard.shunt(ERROR_RULE);
        }
        let mut rule = yard.terminals.pop().unwrap();
        if rule.kind == LexemeKind::Error && rule.children[0].kind == LexemeKind::Error {
            rule.children.pop().unwrap()
        } else {
            rule
        }
    }
}
