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
fn edits_fuzz01() {
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
fn edits_fuzz02() {
    test_edits!(
        [" :=", (0, 0), (0, 0)]
        ["\n", (0, 0), (0, 0)]
        ["*", (0, 0), (0, 0)]
        [":=", (1, 0), (0, 0)]
    );
}

#[test]
fn property_fuzz01() {
    let one_shot = test_expr!("0((:0f(f");

    let incremental = test_edits!(
        ["0f(f", (0, 0), (0, 0)]
        ["0((:", (0, 0), (0, 0)]
    );

    assert_eq!(one_shot, incremental);
}

#[test]
fn property_fuzz02() {
    let one_shot = test_expr!("/(;");

    let incremental = test_edits!(
        ["(;", (0, 0), (0, 0)]
        ["/", (0, 0), (0, 0)]
    );

    assert_eq!(one_shot, incremental);
}

#[test]
fn property_fuzz03() {
    let one_shot = test_expr!("^ <<");

    let incremental = test_edits!(
        [" <<", (0, 0), (0, 0)]
        ["^", (0, 0), (0, 0)]
    );

    println!("{:#}", one_shot);
    println!("{:#}", incremental);
    assert_eq!(one_shot, incremental);
}
