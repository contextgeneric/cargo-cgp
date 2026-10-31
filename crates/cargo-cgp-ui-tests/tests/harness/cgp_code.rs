//! Tests for the form of a `CGP-Exxx` code ([`cargo_cgp_ui_tests::cgp_code`]).

use cargo_cgp_ui_tests::cgp_code::cgp_code_at;
use cargo_cgp_ui_tests::directives::parse;
use cargo_cgp_ui_tests::passes::cgp_codes;

// Property: for all text, `cgp_code_at` gives `CGP-E` and three digits when text starts with them,
// and nothing otherwise.
#[test]
fn a_code_is_the_prefix_and_three_digits() {
    assert_eq!(cgp_code_at("CGP-E101"), Some("CGP-E101"));
    assert_eq!(cgp_code_at("CGP-E101] missing"), Some("CGP-E101"));
    assert_eq!(cgp_code_at("CGP-E10"), None);
    assert_eq!(cgp_code_at("CGP-E1x1"), None);
    assert_eq!(cgp_code_at("CGP-E1é"), None);
    assert_eq!(cgp_code_at("E101"), None);
}

// Property (assumed from `cgp_codes`' callers): a fourth digit does not stop the output's reader,
// which reads the code the text starts with.
#[test]
fn the_output_reader_stops_after_three_digits() {
    assert_eq!(cgp_codes("[CGP-E1234]"), ["CGP-E123"]);
}

// Property: for all directive values, `//@codes:` takes a code only when the whole word is one.
#[test]
#[should_panic(expected = "`//@codes:` takes")]
fn the_directive_rejects_a_fourth_digit() {
    parse("//@codes: CGP-E1234\n", "fixture");
}
