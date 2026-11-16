//! Tests for the canonical order of diagnostics ([`cargo_cgp_error_processing::order`]). The
//! reference model of a span is its pair of offsets, compared inline rather than through
//! `ByteRange`'s own methods.

use std::cmp::Ordering;

use cargo_cgp_error_processing::byte_range::ByteRange;
use cargo_cgp_error_processing::order::{
    OrderKey, SpanPlace, Spanless, preferred_first, sort_by_order_key,
};

fn key(file: &str, lo: u32, hi: u32, code: Option<&str>, message: &str) -> OrderKey {
    OrderKey {
        place: Some(SpanPlace {
            file: file.to_owned(),
            range: ByteRange { lo, hi },
        }),
        code: code.map(str::to_owned),
        message: message.to_owned(),
        detail: Vec::new(),
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
        detail: Vec::new(),
    }
}

fn with_detail(key: OrderKey, detail: &[&str]) -> OrderKey {
    OrderKey {
        detail: detail.iter().map(|part| part.to_string()).collect(),
        ..key
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
//   message, detail), with every key lacking a span equal and last.
// Property: for all keys over small spans, two codes, a detail, and a missing span, `cmp` is
//   antisymmetric and transitive.
#[test]
fn the_order_is_total_on_small_keys() {
    let mut keys: Vec<OrderKey> = small_spans()
        .into_iter()
        .flat_map(|(lo, hi)| {
            [
                key("main.rs", lo, hi, Some("E0271"), "m"),
                key("main.rs", lo, hi, Some("E0277"), "m"),
                with_detail(key("main.rs", lo, hi, Some("E0277"), "m"), &["child"]),
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

// Concern: two diagnostics at one span with one code and message that print differently, such as
//   two `E0207`s of which one carries the macro's span label, would tie, so a stable sort would
//   keep them in arrival order, which varies under threads.
// Handled: the detail, the rest of the diagnostic's content, breaks the tie; it is compared last.
// Property: for all keys with one place, code, and message, the detail decides, and it never
//   overrides an earlier field.
#[test]
fn the_detail_breaks_a_tie_and_only_a_tie() {
    let plain = at(3, 9);
    let labelled = with_detail(at(3, 9), &["label true main.rs:0-9 unconstrained"]);
    assert!(plain < labelled);
    assert_ne!(plain, labelled);
    assert!(with_detail(at(3, 9), &["b"]) > with_detail(at(3, 9), &["a", "z"]));
    assert!(with_detail(at(3, 9), &["z"]) < with_detail(at(0, 10), &["a"]));
    assert!(with_detail(key("main.rs", 3, 9, Some("E0277"), "a"), &["z"]) < at(3, 9));
}

// Property: two keys without a span stay equal whatever their detail, so rustc's closing lines
//   keep their order.
#[test]
fn the_detail_leaves_spanless_keys_equal() {
    assert_eq!(
        with_detail(spanless(None, "a"), &["x"]),
        with_detail(spanless(None, "a"), &["y"])
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
    let sorted: Vec<&str> =
        sort_by_order_key(items, |(_, key)| key.clone(), Spanless::InArrivalOrder)
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

/// The keys `the_order_is_total_on_small_keys` compares, with three spanless keys that differ in
/// their code, their message, or their detail.
fn small_keys() -> Vec<OrderKey> {
    let mut keys: Vec<OrderKey> = small_spans()
        .into_iter()
        .flat_map(|(lo, hi)| {
            [
                key("main.rs", lo, hi, Some("E0277"), "m"),
                with_detail(key("main.rs", lo, hi, Some("E0277"), "m"), &["child"]),
            ]
        })
        .collect();
    keys.push(spanless(None, "a"));
    keys.push(spanless(Some("E0277"), "a"));
    keys.push(with_detail(spanless(Some("E0277"), "a"), &["child"]));
    keys
}

// Property: the postcondition of `OrderKey::cmp_by_content`, for all small keys: it agrees with
//   `cmp` whenever one key has a span, and orders two spanless keys by (code, message, detail).
#[test]
fn comparing_by_content_differs_from_cmp_only_between_spanless_keys() {
    let keys = small_keys();
    for a in &keys {
        for b in &keys {
            let expected = match (&a.place, &b.place) {
                (None, None) => {
                    (&a.code, &a.message, &a.detail).cmp(&(&b.code, &b.message, &b.detail))
                }
                _ => a.cmp(b),
            };
            assert_eq!(a.cmp_by_content(b), expected, "{a:?} {b:?}");
        }
    }
}

// Concern: a comparison that orders spanless keys by content as well could be intransitive across
//   the spanned and spanless keys, so a sort's result would depend on its input order.
// Handled: every spanned key comes before every spanless one, and each side is a lexicographic
//   order.
// Property: for all small keys, `cmp_by_content` is antisymmetric and transitive.
#[test]
fn comparing_by_content_is_total_on_small_keys() {
    let keys = small_keys();
    for a in &keys {
        for b in &keys {
            assert_eq!(
                a.cmp_by_content(b),
                b.cmp_by_content(a).reverse(),
                "{a:?} {b:?}"
            );
            for c in &keys {
                if a.cmp_by_content(b).is_le() && b.cmp_by_content(c).is_le() {
                    assert!(a.cmp_by_content(c).is_le(), "{a:?} {b:?} {c:?}");
                }
            }
        }
    }
}

/// Every order of `items`.
fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.is_empty() {
        return vec![Vec::new()];
    }
    (0..items.len())
        .flat_map(|i| {
            let mut rest = items.to_vec();
            let first = rest.remove(i);
            permutations(&rest).into_iter().map(move |mut tail| {
                tail.insert(0, first.clone());
                tail
            })
        })
        .collect()
}

// Concern: rustc's threads can emit two diagnostics without a span in either order, and a sort
//   that keeps such diagnostics in arrival order would then process and print them differently
//   on each run.
// Handled: `Spanless::ByContent` orders them by their content.
// Property: for every arrival order of three spanless items and a spanned one,
//   `sort_by_order_key` with `Spanless::ByContent` gives one result, the spanned item first and
//   the spanless ones by code, then message.
#[test]
fn sorting_by_content_gives_one_order_for_every_arrival_order() {
    let items = [
        ("overflow", spanless(Some("E0275"), "overflow")),
        ("cycle", spanless(Some("E0391"), "cycle")),
        ("plain", spanless(None, "unlocated")),
        ("located", at(4, 6)),
    ];
    for arrived in permutations(&items) {
        let sorted: Vec<&str> =
            sort_by_order_key(arrived.clone(), |(_, key)| key.clone(), Spanless::ByContent)
                .into_iter()
                .map(|(name, _)| name)
                .collect();
        let names: Vec<&str> = arrived.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            sorted,
            ["located", "plain", "overflow", "cycle"],
            "arrived as {names:?}"
        );
    }
}

// Property: the postcondition of `preferred_first`, for every arrangement of two preferred items
//   among two others: the preferred come first, and each part keeps its order.
#[test]
fn preferred_items_come_first_and_each_part_keeps_its_order() {
    let arrangements: [[&str; 4]; 6] = [
        ["check a", "check b", "call a", "call b"],
        ["call a", "call b", "check a", "check b"],
        ["call a", "check a", "call b", "check b"],
        ["check a", "call a", "check b", "call b"],
        ["call a", "check a", "check b", "call b"],
        ["check a", "call a", "call b", "check b"],
    ];
    for items in arrangements {
        let out = preferred_first(items.to_vec(), |item| item.starts_with("check"));
        assert_eq!(
            out,
            ["check a", "check b", "call a", "call b"],
            "for {items:?}"
        );
    }
}
