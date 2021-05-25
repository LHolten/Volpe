mod common;
use common::*;
use pretty_assertions::assert_eq;

#[test]
fn simple() {
    test_edits!(
        ["a", (0, 0), (0, 0)]
        [" ", (0, 1), (0, 0)]
        ["+", (0, 2), (0, 0)]
        [" ", (0, 3), (0, 0)]
        ["b", (0, 3), (0, 0)]
        ["(", (0, 0), (0, 0)]
        [" ", (0, 2), (0, 0)]
        [")", (0, 4), (0, 0)]
    );
}

#[test]
fn found_by_fuzz() {
    test_edits!(
        ["A.AA", (0, 0), (0, 0)]
        [" .A ", (0, 0), (0, 0)]
        ["+", (0, 0), (0, 0)]
        [" A", (0, 0), (0, 0)]
        ["A&", (0, 0), (0, 0)]
        [".,", (0, 2), (0, 0)]
        ["A", (0, 14), (0, 0)]
    );
}

#[test]
fn incremental_vs_one_shot_fuzz01() {
    let one_shot = test_expr!("0((:0f(f");

    let incremental = test_edits!(
        ["0f(f", (0, 0), (0, 0)]
        ["0((:", (0, 0), (0, 0)]
    );

    assert_eq!(one_shot, incremental);
}

#[test]
fn incremental_vs_one_shot_fuzz02() {
    let one_shot = test_expr!("/(;");

    let incremental = test_edits!(
        ["(;", (0, 0), (0, 0)]
        ["/", (0, 0), (0, 0)]
    );

    assert_eq!(one_shot, incremental);
}
