//! Tests for which qualified names stay qualified.

use cargo_cgp_error_processing::AmbiguousNames;

// Property: for all paths rustc prints for a glob-imported item, `keeps_qualifier` is false, and a
// tail starting at a root name names another item.
#[test]
fn a_glob_item_reads_bare_from_each_path_rustc_prints() {
    // rustc prints a glob-imported item from any enclosing module whose name it trims, so each such
    // tail names the item; a tail starting at a crate-root name is the root item's own path.
    let ambiguous = AmbiguousNames::new(
        [(
            "EvalAdd".to_owned(),
            vec!["providers::eval::add::EvalAdd".to_owned()],
        )],
        ["eval".to_owned()],
    );
    assert!(!ambiguous.keeps_qualifier("providers::eval::add::EvalAdd", "EvalAdd"));
    assert!(!ambiguous.keeps_qualifier("add::EvalAdd", "EvalAdd"));
    assert!(ambiguous.keeps_qualifier("eval::add::EvalAdd", "EvalAdd"));
    assert!(ambiguous.keeps_qualifier("other::add::EvalAdd", "EvalAdd"));
}
