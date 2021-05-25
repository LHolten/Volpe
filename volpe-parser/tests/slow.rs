mod common;
use common::*;

#[test]
fn brackets() {
    with_timeout(
        || {
            test_expr!("(((");
        },
        std::time::Duration::from_secs(5),
    );
}
