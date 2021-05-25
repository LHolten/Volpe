mod common;
use common::*;

#[test]
fn basic_symbols() {
    test_expr!("");
    test_expr!("a");
    test_expr!("0");
    test_expr!("+");
    test_expr!("-");
    test_expr!("*");
    test_expr!("/");
    test_expr!("%");
    test_expr!("=");
    test_expr!(":=");
    test_expr!(":");
    test_expr!(",");
    test_expr!("=>");
    test_expr!("(");
    test_expr!(")");
    test_expr!("()");
    test_expr!("{");
    test_expr!("}");
    test_expr!("{}");
}

#[test]
fn longer_expressions() {
    test_expr!("f = x.(x + 1)");
    test_expr!("(a.b.a + b) 1 2");
    test_expr!("{1 2} x.y.{y x}");
    test_expr!("a * b + c / d");
}
