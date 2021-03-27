use crate::{
    combinators::{Alt, Id, LexemeP, Many0, Many1, NotOpt, Opt, Pair, RuleP, Separated},
    lexeme_kind::LexemeKind as L,
    syntax::RuleKind,
    tracker::{TFunc, TInput, TResult},
};

pub struct FileP;

impl TFunc for FileP {
    fn parse(mut t: TInput) -> TResult {
        t = Opt::<LexemeP<{ L::Start.mask() }>>::parse(t)?;
        t = Expr::parse(t)?;
        Many0::<Pair<LexemeP<{ L::RBrace.mask() | L::RCurlyBrace.mask() | L::Comma.mask() }>, Expr>>::parse(t)
    }
}

type Expr = Alt<RuleP<ExprInner, { RuleKind::Expr as usize }>, Alt<Stmt, App>>;
type Semi = LexemeP<{ L::Semicolon.mask() }>;

pub struct ExprInner;

impl TFunc for ExprInner {
    fn parse(mut t: TInput) -> TResult {
        t = App::parse(t)?;
        t = LexemeP::<{ L::Assign.mask() | L::Ite.mask() }>::parse(t)?;
        t = App::parse(t)?;
        t = Opt::<Semi>::parse(t)?;
        Expr::parse(t)
    }
}

type Stmt = RuleP<StmtInner, { RuleKind::Stmt as usize }>;
type MultiAssign = Many1<Pair<LexemeP<{ L::MultiAssign as usize }>, App>>;

pub struct StmtInner;

impl TFunc for StmtInner {
    fn parse(mut t: TInput) -> TResult {
        t = App::parse(t)?;
        t = Alt::<Pair<MultiAssign, Opt<Semi>>, Semi>::parse(t)?;
        Expr::parse(t)
    }
}

type App = Alt<RuleP<Separated<NotOpt<Func>, Id>, { RuleKind::App as usize }>, Func>;

type Func = Alt<RuleP<Separated<Or, LexemeP<{ L::Func.mask() }>>, { RuleKind::Func as usize }>, Or>;

type Or = Alt<RuleP<Separated<And, LexemeP<{ L::Or.mask() }>>, { RuleKind::Or as usize }>, And>;

type And = Alt<RuleP<Separated<Op1, LexemeP<{ L::And.mask() }>>, { RuleKind::And as usize }>, Op1>;

const TAG1: usize = L::Equals.mask()
    | L::UnEquals.mask()
    | L::Less.mask()
    | L::Greater.mask()
    | L::GreaterEqual.mask()
    | L::LessEqual.mask();
type Op1 = Alt<RuleP<Separated<Op2, LexemeP<{ TAG1 }>>, { RuleKind::Op1 as usize }>, Op2>;

const TAG2: usize =
    L::Plus.mask() | L::Minus.mask() | L::BitOr.mask() | L::BitShl.mask() | L::BitShr.mask();
type Op2 = Alt<RuleP<Separated<Op3, LexemeP<{ TAG2 }>>, { RuleKind::Op2 as usize }>, Op3>;

const TAG3: usize = L::Mul.mask() | L::Div.mask() | L::Mod.mask() | L::BitAnd.mask();
type Op3 = Alt<RuleP<Separated<Term, LexemeP<{ TAG3 }>>, { RuleKind::Op3 as usize }>, Term>;

type Term = Opt<Alt<LexemeP<{ L::Num.mask() | L::Ident.mask() }>, Alt<Block, Tuple>>>;

pub struct Block;
impl TFunc for Block {
    fn parse(mut t: TInput) -> TResult {
        t = LexemeP::<{ L::LBrace.mask() }>::parse(t)?;
        t = Expr::parse(t)?;
        Opt::<LexemeP<{ L::RBrace.mask() }>>::parse(t)
    }
}

type Tuple = RuleP<TupleInner, { RuleKind::Tuple as usize }>;

pub struct TupleInner;
impl TFunc for TupleInner {
    fn parse(mut t: TInput) -> TResult {
        t = LexemeP::<{ L::LCurlyBrace.mask() }>::parse(t)?;
        t = Separated::<TupleLine, Opt<LexemeP<{ L::Comma.mask() }>>>::parse(t)?;
        Opt::<LexemeP<{ L::RCurlyBrace.mask() }>>::parse(t)
    }
}

type TupleLine = Pair<App, Opt<Pair<LexemeP<{ L::Colon.mask() }>, App>>>;

#[cfg(test)]
mod tests {
    use crate::offset::Offset;
    use crate::packrat::Parser;

    macro_rules! test_expr {
        ($s:literal) => {
            let mut pos = Parser::default();
            pos.parse($s, Offset::default(), Offset::default());
        };
    }

    #[test]
    fn functions() {
        test_expr!("hello world");
        // test_expr!("[1, 2, 3; 4, 5, 6] 10 (cool thing)");
        test_expr!("(a = 1; b = 2; add a b)");
        // test_expr!(
        //     "my_object = {
        //         alpha : something,
        //         beta : 3404,
        //     }; my_object"
        // );
        test_expr!("a.b.(add a b) 10 20");
        test_expr!("{1, 2, 3}");
        test_expr!("{1}");
        test_expr!("{}");
        test_expr!("1 > 2 => {}");
        // test_expr!("1 /* /* wow */ cool */ > 2 // hello");
    }
}
