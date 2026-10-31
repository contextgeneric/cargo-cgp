//! Tests for reading text outside string literals, directly and through its three callers.

use cargo_cgp_error_processing::{
    code_chars, render_variant, scan_head_tail, string_literal_end, strip_module_paths,
};

// Property: for all text, a `\"` inside a string literal does not end it, and a `\\` before a quote
// leaves that quote closing it, for each reader of a literal.
#[test]
fn an_escape_decides_where_a_literal_ends() {
    assert_eq!(
        strip_module_paths(r#""a::B\"c::D" e::F"#).as_deref(),
        Some(r#""a::B\"c::D" F"#)
    );
    assert_eq!(
        strip_module_paths(r#""a::B\\" e::F"#).as_deref(),
        Some(r#""a::B\\" F"#)
    );
    assert_eq!(
        scan_head_tail(r#"Name<"a\\">, Nil>"#),
        Some((r#"Name<"a\\">"#, " Nil", 17))
    );
    assert_eq!(
        render_variant("V", Some(r#"Struct!(Symbol!("\")"))"#)),
        r#"V(Symbol!("\")"))"#
    );
}

// Property: for all text whose last string literal does not close, the rest of the text is inside
// it, so nothing in it is read as code.
#[test]
fn an_unclosed_literal_runs_to_the_end() {
    assert_eq!(strip_module_paths(r#""a::B"#), None);
    assert_eq!(scan_head_tail(r#"u8, "Nil>"#), None);
    // The `(` is inside the literal, so the call's own `)` still balances the body.
    assert_eq!(render_variant("V", Some(r#"Struct!("a(b)"#)), r#"V("a(b)"#);
}

// Property: for all text and `open` at a `"`, `string_literal_end` is just past the first unescaped
// quote after `open`, or the end of the text.
#[test]
fn a_literal_ends_just_past_its_closing_quote() {
    assert_eq!(string_literal_end(r#"x"ab"y"#, 1), 5);
    assert_eq!(string_literal_end(r#""é"y"#, 0), 4);
    assert_eq!(string_literal_end(r#""a\"b"y"#, 0), 6);
    assert_eq!(string_literal_end(r#""a"#, 0), 2);
    assert_eq!(string_literal_end(r#"""#, 0), 1);
}

// Property: for all text, `code_chars` gives exactly the characters outside string literals, each
// with its own byte offset.
#[test]
fn code_chars_leaves_out_literals_and_their_quotes() {
    let found: Vec<(usize, char)> = code_chars(r#"é"a"b"c"#).collect();
    assert_eq!(found, [(0, 'é'), (5, 'b')]);
}
