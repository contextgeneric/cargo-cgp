//! Tests for the split of a spine cell into its head and tail.

use cargo_cgp_error_processing::scan_head_tail;

// Property: for all text after a cell's opening token, `scan_head_tail` splits at the first
// top-level comma and ends just past the matching `>`, whatever nesting and string literals hold,
// and gives `None` when either is missing.
#[test]
fn scan_head_tail_splits_at_the_top_level_comma() {
    // The scan reads the text after a cell's opening token; nesting and string literals hide their
    // commas and closing brackets, and the offset points just past the cell's own `>`.
    let after = "Symbol<3, Chars<'a', Nil>>, Cons<(u8, u16), Nil>> rest";
    let (head, tail, end) = scan_head_tail(after).unwrap();
    assert_eq!(head, "Symbol<3, Chars<'a', Nil>>");
    assert_eq!(tail, " Cons<(u8, u16), Nil>");
    assert_eq!(&after[end..], " rest");

    let (head, tail, _) = scan_head_tail(r#"Name<"a, >b">, Nil>"#).unwrap();
    assert_eq!((head, tail), (r#"Name<"a, >b">"#, " Nil"));

    // An escaped quote does not end the string literal.
    let (head, tail, _) = scan_head_tail(r#"Name<"a\", >b">, Nil>"#).unwrap();
    assert_eq!((head, tail), (r#"Name<"a\", >b">"#, " Nil"));

    // No top-level comma, or no matching close, declines.
    assert_eq!(scan_head_tail("Nil>"), None);
    assert_eq!(scan_head_tail("u8, Nil"), None);
}
