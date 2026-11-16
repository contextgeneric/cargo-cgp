//! Tests for the canonical order of diagnostics ([`cargo_cgp_error_processing::order`]). The
//! reference model of a span is its pair of offsets, compared inline rather than through
//! `ByteRange`'s own methods.

use std::cmp::Ordering;

use cargo_cgp_error_processing::byte_range::ByteRange;
use cargo_cgp_error_processing::order::{OrderKey, SpanPlace, sort_by_order_key};

fn key(file: &str, lo: u32, hi: u32, code: Option<&str>, message: &str) -> OrderKey {
    OrderKey {
        place: Some(SpanPlace {
            file: file.to_owned(),
            range: ByteRange { lo, hi },
        }),
        code: code.map(str::to_owned),
        message: message.to_owned(),
    }
}

fn at(lo: u32, hi: u32) -> OrderKey {
    key("main.rs", lo, hi, Some("E0277"), "m")
}

fn spanless(code: Option<&str>, message: &str) -> OrderKey {
    OrderKey {
        place: None,
        code: code.map(str::to_owned),
        message: message.to_owned(),
    }
}

/// Every span within the first five bytes, the empty ones included.
fn small_spans() -> Vec<(u32, u32)> {
    (0..=4)
        .flat_map(|lo| (lo..=4).map(move |hi| (lo, hi)))
        .collect()
}

// Property: for all spans a and b in one file with b containing a (b.lo <= a.lo && a.hi <= b.hi)
//   and a != b, the key at a comes before the key at b: the inner report leads.
#[test]
fn an_inner_span_comes_before_the_span_around_it_on_small_spans() {
    for a in small_spans() {
        for b in small_spans() {
            let contains = b.0 <= a.0 && a.1 <= b.1;
            if contains && a != b {
                assert!(at(a.0, a.1) < at(b.0, b.1), "{a:?} inside {b:?}");
            }
        }
    }
}

// Property: for all spans a and b in one file where a ends before b starts (a.hi <= b.lo) and
//   neither contains the other, the key at a comes first: disjoint reports follow the source.
#[test]
fn a_span_before_another_comes_first_on_small_spans() {
    for a in small_spans() {
        for b in small_spans() {
            let nested = (b.0 <= a.0 && a.1 <= b.1) || (a.0 <= b.0 && b.1 <= a.1);
            if a.1 <= b.0 && !nested {
                assert!(at(a.0, a.1) < at(b.0, b.1), "{a:?} before {b:?}");
            }
        }
    }
}

// Concern: a comparison built from several tie-breaks could be intransitive, which would let a
//   sort's result depend on the order of its input.
// Handled: the comparison is a lexicographic order over (file, end, reversed start, code,
//   message), with every key lacking a span equal and last.
// Property: for all keys over small spans, two codes, and a missing span, `cmp` is antisymmetric
//   and transitive.
#[test]
fn the_order_is_total_on_small_keys() {
    let mut keys: Vec<OrderKey> = small_spans()
        .into_iter()
        .flat_map(|(lo, hi)| {
            [
                key("main.rs", lo, hi, Some("E0271"), "m"),
                key("main.rs", lo, hi, Some("E0277"), "m"),
            ]
        })
        .collect();
    keys.push(spanless(None, "a"));
    keys.push(spanless(Some("E0277"), "b"));
    for a in &keys {
        for b in &keys {
            assert_eq!(a.cmp(b), b.cmp(a).reverse(), "{a:?} {b:?}");
            for c in &keys {
                if a <= b && b <= c {
                    assert!(a <= c, "{a:?} {b:?} {c:?}");
                }
            }
        }
    }
}

// Property: for all keys at one span, the code decides, then the message.
#[test]
fn one_span_orders_by_code_then_message() {
    let span = |code: Option<&str>, message: &str| key("main.rs", 3, 9, code, message);
    assert!(span(None, "z") < span(Some("E0271"), "a"));
    assert!(span(Some("E0271"), "z") < span(Some("E0277"), "a"));
    assert!(span(Some("E0277"), "a") < span(Some("E0277"), "b"));
    assert_eq!(
        span(Some("E0277"), "a").cmp(&span(Some("E0277"), "a")),
        Ordering::Equal
    );
}

// Property: for all keys in two files, the file name decides, whatever the offsets.
#[test]
fn the_file_name_decides_before_the_offsets() {
    assert!(key("a.rs", 90, 99, None, "m") < key("b.rs", 0, 1, None, "m"));
    assert!(key("src/lib.rs", 0, 1, None, "m") > key("src/a/mod.rs", 50, 60, None, "m"));
}

// Property: a key with a span comes before every key without one, and two keys without a span
//   are equal whatever their codes and messages.
#[test]
fn a_spanless_key_comes_last_and_ties_with_another() {
    assert!(at(900, 999) < spanless(None, "a"));
    assert_eq!(spanless(Some("E0277"), "b"), spanless(None, "a"));
}

// Property: the postcondition of `sort_by_order_key`: the result is sorted, and items with equal
//   keys keep their order, here the spanless items and two items at one span.
#[test]
fn sorting_keeps_equal_keys_in_their_order() {
    let items: Vec<(&str, OrderKey)> = vec![
        ("footer", spanless(None, "For more information")),
        ("outer", at(0, 10)),
        ("first at 4..6", at(4, 6)),
        ("summary", spanless(Some("E0277"), "aborting")),
        ("second at 4..6", at(4, 6)),
        ("before", at(0, 2)),
    ];
    let sorted: Vec<&str> = sort_by_order_key(items, |(_, key)| key.clone())
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(
        sorted,
        [
            "before",
            "first at 4..6",
            "second at 4..6",
            "outer",
            "footer",
            "summary"
        ]
    );
}
