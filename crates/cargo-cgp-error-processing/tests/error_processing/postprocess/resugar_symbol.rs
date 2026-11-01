//! Tests for the `Symbol!` resugaring.
//!
//! Each transform reads plain text and gives `Some` when it changed it, so it is driven directly
//! over the case under test, with no diagnostic wrapper and no compiler.

use cargo_cgp_error_processing::resugar_symbol;

// Property: for all exact `Symbol<LEN, spine>` matches, only the symbol changes.
#[test]
fn resugars_an_exact_symbol() {
    // `height`: six ASCII characters, so the length is 6. The surrounding `HasField<…>`
    // must survive with its own closing `>`.
    assert_eq!(
        resugar_symbol(
            "HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>>"
        )
        .as_deref(),
        Some("HasField<Symbol!(\"height\")>"),
    );
}

// Property: an exact match of length zero resugars to the empty string.
#[test]
fn resugars_the_empty_symbol() {
    assert_eq!(
        resugar_symbol("Symbol<0, Nil>").as_deref(),
        Some("Symbol!(\"\")")
    );
}

// Property: a `LEN` different from the decoded byte length is not an exact match.
#[test]
fn resugar_skips_a_wrong_length() {
    // Declared length 3 but only two characters: not an exact match, so left alone.
    assert_eq!(
        resugar_symbol("Symbol<3, Chars<'x', Chars<'y', Nil>>>"),
        None
    );
}

// Property: a `Symbol` whose payload is not a `Chars` spine is not an exact match.
#[test]
fn resugar_skips_a_foreign_symbol() {
    // A type named `Symbol` with a non-`Chars` payload must not be rewritten.
    assert_eq!(resugar_symbol("Symbol<2, SomeOtherType>"), None);
}

// Property: for all text with no `Symbol<`, `resugar_symbol` gives `None`.
#[test]
fn resugar_leaves_other_symbol_uses_alone() {
    assert_eq!(
        resugar_symbol("the trait `Symbolic` is not implemented"),
        None
    );
}
