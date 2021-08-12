use std::rc::Rc;

use string_interner::{DefaultSymbol, StringInterner};
use void::{ResultVoidExt, Void};

use crate::{
    grammar::RuleKind,
    lexeme_kind::LexemeKind,
    syntax::{Lexeme, Syntax},
};

#[derive(Clone, PartialEq)]
pub enum Const {
    BuiltIn(LexemeKind),
    Custom(DefaultSymbol),
}

// this type can only hold the desugared version of the source code
#[derive(Clone, PartialEq)]
pub enum Simple {
    Abs(bool, Rc<Simple>), //strict
    App([Rc<Simple>; 2]),
    Const(Const),
    Ident(usize),
    Num(i32),
}

impl Simple {
    pub fn as_ident(&self) -> usize {
        match self {
            Simple::Ident(val) => *val,
            _ => unreachable!(),
        }
    }

    pub fn replace(&self, depth: usize, val: &Simple) -> Self {
        match self {
            Simple::Abs(strict, func) => Simple::Abs(*strict, func.replace(depth + 1, val).into()),
            Simple::App([func, arg]) => Simple::App([
                func.replace(depth, val).into(),
                arg.replace(depth, val).into(),
            ]),
            Simple::Ident(index) => {
                if index == &depth {
                    val.clone()
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }
}

#[derive(Default)]
pub struct ASTBuilder {
    interner: StringInterner,
    arg_ident: Vec<DefaultSymbol>,
}

impl ASTBuilder {
    pub fn convert<'a>(&mut self, syntax: impl AsRef<Syntax<'a, Void>>) -> Simple {
        match syntax.as_ref() {
            Syntax::Operator { operator, operands } => {
                if matches!(
                    operator,
                    Some(Lexeme {
                        kind: LexemeKind::Abs | LexemeKind::AbsStrict,
                        ..
                    })
                ) {
                    let ident = match operands[0].as_ref() {
                        Syntax::Terminal(Ok(Lexeme { text, .. })) => {
                            self.interner.get_or_intern(text)
                        }
                        _ => todo!(),
                    };

                    self.arg_ident.push(ident);
                    let second = self.convert(&operands[1]).into();
                    self.arg_ident.pop();

                    if operator.unwrap().kind == LexemeKind::Abs {
                        Simple::Abs(false, second)
                    } else {
                        Simple::Abs(true, second)
                    }
                } else {
                    let mut item = self.convert(&operands[0]).into();

                    if let Some(lexeme) = operator {
                        lexeme.kind.assert_simple_operator();
                        item =
                            Simple::App([item, Simple::Const(Const::BuiltIn(lexeme.kind)).into()])
                                .into();
                    }
                    Simple::App([item, self.convert(&operands[1]).into()])
                }
            }
            Syntax::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => inner
                    .as_ref()
                    .map(|inner| self.convert(inner))
                    .unwrap_or_else(|| Simple::Abs(false, Rc::new(Simple::Ident(0)))),
                LexemeKind::LCurlyBracket => Simple::Abs(false, {
                    self.arg_ident.push(self.interner.get_or_intern_static("$"));
                    let mut func_call = Simple::Ident(0);
                    if let Some(mut item) = inner.as_ref() {
                        while let Syntax::Operator {
                            operator:
                                Some(Lexeme {
                                    kind: LexemeKind::Comma,
                                    ..
                                }),
                            operands,
                        } = item.as_ref()
                        {
                            func_call =
                                Simple::App([func_call.into(), self.convert(&operands[0]).into()]);
                            item = &operands[1]
                        }
                        func_call = Simple::App([func_call.into(), self.convert(item).into()]);
                    }
                    self.arg_ident.pop().unwrap();
                    func_call.into()
                }),
                _ => unreachable!(),
            },
            Syntax::Terminal(l) => {
                let lexeme = l.void_unwrap();
                let symbol = self.interner.get_or_intern(lexeme.text);
                match lexeme.kind {
                    LexemeKind::Ident => Simple::Ident({
                        self.arg_ident
                            .iter()
                            .rev()
                            .position(|ident| ident == &symbol)
                            .unwrap()
                    }),
                    LexemeKind::Const => Simple::Const(Const::Custom(symbol)),
                    LexemeKind::Num => Simple::Num(lexeme.text.parse().unwrap()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl LexemeKind {
    pub fn assert_simple_operator(&self) {
        assert!(matches!(self.rule_kind(), RuleKind::Operator));
        assert!(!matches!(
            self,
            LexemeKind::Semicolon | LexemeKind::Assign | LexemeKind::Comma
        ))
    }
}
