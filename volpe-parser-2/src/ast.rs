use std::{cmp::Ordering, rc::Rc};

use string_interner::{DefaultSymbol, StringInterner};
use void::{ResultVoidExt, Void};

use crate::{
    lexeme_kind::LexemeKind,
    stack_list::StackList,
    syntax::{Lexeme, Syntax},
};

// this type can only hold the desugared version of the source code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Simple {
    Abs(bool, Rc<Simple>), //strict
    App([Rc<Simple>; 2]),
    Const(DefaultSymbol),
    Ident(usize),
    Num(i32),
    Case(DefaultSymbol, Rc<Simple>),
    Bot,
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
            Simple::Ident(index) => match index.cmp(&depth) {
                Ordering::Less => self.clone(),
                Ordering::Equal => val.clone(),
                Ordering::Greater => Simple::Ident(index - 1),
            },
            Simple::Case(symbol, body) => Simple::Case(*symbol, body.replace(depth, val).into()),
            _ => self.clone(),
        }
    }

    pub fn strict_len(&self) -> usize {
        match self {
            Simple::Abs(_, func) => func.strict_len().saturating_sub(1),
            Simple::App([func, arg]) => func.strict_len().max(arg.strict_len()),
            Simple::Ident(i) => i + 1,
            _ => 0,
        }
    }
}

#[derive(Default)]
pub struct ASTBuilder {
    interner: StringInterner,
}

impl ASTBuilder {
    pub fn convert<'a>(
        &mut self,
        env: StackList<DefaultSymbol>,
        syntax: impl AsRef<Syntax<'a, Void>>,
    ) -> Simple {
        match syntax.as_ref() {
            Syntax::Operator { operator, operands } => {
                if let Some(operator) = operator {
                    match operator.kind {
                        LexemeKind::Semicolon => Simple::App([
                            self.convert(env, &operands[0]).into(),
                            self.convert(env, &operands[1]).into(),
                        ]),
                        LexemeKind::Assign => {
                            let ident = match operands[0].as_ref() {
                                Syntax::Terminal(Ok(Lexeme { text, .. })) => {
                                    self.interner.get_or_intern(text)
                                }
                                _ => todo!(),
                            };

                            match operands[1].as_ref() {
                                Syntax::Operator { operator, operands } => {
                                    assert_eq!(operator.unwrap().kind, LexemeKind::Semicolon);

                                    let func = Simple::Abs(
                                        false,
                                        self.convert(env.push(&ident), &operands[1]).into(),
                                    );
                                    Simple::App([
                                        func.into(),
                                        self.convert(env, &operands[0]).into(),
                                    ])
                                }
                                _ => unreachable!(),
                            }
                        }
                        LexemeKind::Abs => {
                            let mut constant = false;
                            let ident = match operands[0].as_ref() {
                                Syntax::Terminal(Ok(Lexeme { text, kind, .. })) => {
                                    if kind == &LexemeKind::Const {
                                        constant = true;
                                    }
                                    self.interner.get_or_intern(text)
                                }
                                _ => todo!(),
                            };
                            let strict = operator.text == ":";

                            if constant {
                                let second = self.convert(env, &operands[1]).into();
                                Simple::Case(ident, second)
                            } else {
                                let second = self.convert(env.push(&ident), &operands[1]).into();
                                Simple::Abs(strict, second)
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let item = self.convert(env, &operands[0]).into();
                    Simple::App([item, self.convert(env, &operands[1]).into()])
                }
            }
            Syntax::Brackets { brackets, inner } => match brackets[0].void_unwrap().kind {
                LexemeKind::LRoundBracket => inner
                    .as_ref()
                    .map(|inner| self.convert(env, inner))
                    .unwrap_or_else(|| Simple::Bot),
                LexemeKind::LCurlyBracket => Simple::Abs(false, {
                    // this is just to increment the env local count
                    let ident = self.interner.get_or_intern_static("$");
                    let env = env.push(&ident);

                    let mut func_call = Simple::Ident(0);
                    if let Some(mut item) = inner.as_ref() {
                        while let Syntax::Operator {
                            operator: None,
                            operands,
                        } = item.as_ref()
                        {
                            func_call = Simple::App([
                                func_call.into(),
                                self.convert(env, &operands[0]).into(),
                            ]);
                            item = &operands[1]
                        }
                        func_call = Simple::App([func_call.into(), self.convert(env, item).into()]);
                    }
                    func_call.into()
                }),
                _ => unreachable!(),
            },
            Syntax::Terminal(l) => {
                let lexeme = l.void_unwrap();
                let symbol = self.interner.get_or_intern(lexeme.text);
                match lexeme.kind {
                    LexemeKind::Ident => Simple::Ident(env.find(&symbol).unwrap()),
                    LexemeKind::Const => Simple::Const(symbol),
                    LexemeKind::Num => Simple::Num(lexeme.text.parse().unwrap()),
                    _ => unreachable!(),
                }
            }
        }
    }
}
