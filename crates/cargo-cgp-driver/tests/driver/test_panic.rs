//! Tests for the panic a UI fixture makes the emitter raise on purpose.

use cargo_cgp_driver::emitter::TestPanic;

#[test]
fn names_each_stage() {
    assert_eq!(TestPanic::parse("transform"), Some(TestPanic::Transform));
    assert_eq!(TestPanic::parse("flush"), Some(TestPanic::Flush));
}

#[test]
fn any_other_value_arms_nothing() {
    assert_eq!(TestPanic::parse(""), None);
    assert_eq!(TestPanic::parse("Transform"), None);
}
