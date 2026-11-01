//! Tests for resolving a fixture's declared auxiliary crates ([`cargo_cgp_ui_tests::aux`]).

use std::path::{Path, PathBuf};

use cargo_cgp_ui_tests::aux;

// Property: for all declared names, `required` gives the ones with a materialized crate in declared
// order, and skips a name with none, so a typo surfaces in the fixture's snapshot.
#[test]
fn required_keeps_declared_order_and_skips_an_unknown_name() {
    let materialized = vec![
        ("b".to_owned(), PathBuf::from("dir-b")),
        ("a".to_owned(), PathBuf::from("dir-a")),
    ];
    let declared = ["a".to_owned(), "typo".to_owned(), "b".to_owned()];
    assert_eq!(
        aux::required(&declared, &materialized),
        [("a", Path::new("dir-a")), ("b", Path::new("dir-b"))]
    );
}
