//! Tests for parsing rustc's trait-bound header.

use cargo_cgp_error_processing::rewrite::parse_trait_bound;

// Property: for all trait-bound headers, `parse_trait_bound` gives the subject, the whole bound, the trait's last
// segment, and the raw generic arguments; any other message gives `None`.
#[test]
fn parses_a_trait_bound_header() {
    // The classification parse the driver uses on its own: subject, whole bound, trait name.
    let parsed = parse_trait_bound(
        "the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied",
    )
    .expect("a trait-bound header must parse");
    assert_eq!(parsed.subject, "Rectangle");
    assert_eq!(
        parsed.bound,
        "Rectangle: CanUseComponent<AreaCalculatorComponent>"
    );
    assert_eq!(parsed.trait_name, "CanUseComponent");

    // A bound without generics parses too, with empty args.
    let parsed = parse_trait_bound("the trait bound `f64: std::cmp::Eq` is not satisfied")
        .expect("an ordinary bound must parse");
    assert_eq!(parsed.subject, "f64");
    assert_eq!(parsed.bound, "f64: std::cmp::Eq");
    assert_eq!(parsed.trait_name, "Eq");
    assert_eq!(parsed.args, "");

    assert!(parse_trait_bound("mismatched types").is_none());
}
