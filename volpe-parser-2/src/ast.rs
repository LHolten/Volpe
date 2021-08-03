use std::rc::Rc;

use string_interner::{DefaultSymbol, StringInterner};
use void::{ResultVoidExt, Void};

use crate::{
    grammar::RuleKind,
    lexeme_kind::LexemeKind,
    syntax::{Lexeme, Syntax},
};

pub enum TerminalKind {
    Const,
    Num,
    Ident,
}

// this type can only hold the desugared version of the source code
pub enum Simple {
    Func([Rc<Simple>; 2]),
    App([Rc<Simple>; 2]),
    Terminal {
        kind: TerminalKind,
        val: DefaultSymbol,
    },
}

impl Simple {
    pub fn as_ident(&self) -> DefaultSymbol {
        match self {
            Simple::Terminal {
                kind: TerminalKind::Ident,
                val,
            } => *val,
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct ASTBuilder(StringInterner);

impl ASTBuilder {
    pub fn convert<'a>(&mut self, syntax: impl AsRef<Syntax<'a, Void>>) -> Simple {
        match syntax.as_ref() {
            Syntax::Operator { operator, operands } => {
                let mut result = self.convert(&operands[0]);
                if let Some(lexeme) = operator {
                    lexeme.kind.assert_simple_operator();
                    result = Simple::App([
                        result.into(),
                        Simple::Terminal {
                            kind: TerminalKind::Const, // all operations become
                            val: self.0.get_or_intern(lexeme.text),
                        }
                        .into(),
                    ])
                }
                Simple::App([result.into(), self.convert(&operands[1]).into()])
            }
            Syntax::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => self.convert(inner),
                LexemeKind::LCurlyBracket => Simple::Func([
                    Simple::Terminal {
                        kind: TerminalKind::Ident,
                        val: self.0.get_or_intern_static("$"),
                    }
                    .into(),
                    {
                        let mut func_call = Simple::Terminal {
                            kind: TerminalKind::Ident,
                            val: self.0.get_or_intern_static("$"),
                        };
                        let mut item = inner;
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
                        func_call
                    }
                    .into(),
                ]),
                _ => unreachable!(),
            },
            Syntax::Terminal(l) => {
                let lexeme = l.void_unwrap();
                Simple::Terminal {
                    kind: match lexeme.kind {
                        LexemeKind::Ident => TerminalKind::Ident,
                        LexemeKind::Const => TerminalKind::Const,
                        LexemeKind::Num => TerminalKind::Num,
                        _ => unreachable!(),
                    },
                    val: self.0.get_or_intern(lexeme.text),
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
