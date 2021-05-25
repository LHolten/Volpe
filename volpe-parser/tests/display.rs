mod common;
use common::*;

#[test]
fn test_display() {
    assert_eq!(
        format!("{}", test_expr!("1 => 2 / 3; 4")),
        "[\n    Expr [\n        Num: \"1 \",\n        Ite: \"=> \",\n        Op3 [\n            Num: \"2 \",\n            Div: \"/ \",\n            Num: \"3\",\n        ],\n        Semicolon: \"; \",\n        Num: \"4\",\n    ],\n]"
    )
}
