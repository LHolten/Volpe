use void::Void;

use crate::syntax::Syntax;

impl<'a> Syntax<'a, ()> {
    // TODO this function needs a lot more checks
    // things like checking that brackets match and operator precedence
    pub fn unwrap(&self) -> Syntax<'a, Void> {
        match self {
            Syntax::Operator { operator, operands } => Syntax::Operator {
                operator: *operator,
                operands: [operands[0].unwrap().into(), operands[1].unwrap().into()],
            },
            Syntax::Brackets { brackets, inner } => Syntax::Brackets {
                brackets: [Ok(brackets[0].unwrap()), Ok(brackets[1].unwrap())],
                inner: inner.unwrap().into(),
            },
            Syntax::Terminal(t) => Syntax::Terminal(Ok(t.unwrap())),
        }
    }
}
