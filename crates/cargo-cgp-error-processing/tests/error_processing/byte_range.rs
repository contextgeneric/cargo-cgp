//! Tests for the anchors' decisions over byte ranges. The reference model of a range is the set of
//! bytes it holds, independent of the comparisons under test.

use std::collections::BTreeSet;

use cargo_cgp_error_processing::byte_range::{
    ByteRange, CaretPlace, Operand, caret_place, is_about_call,
};

fn range(lo: u32, hi: u32) -> ByteRange {
    ByteRange { lo, hi }
}

fn bytes(range: ByteRange) -> BTreeSet<u32> {
    (range.lo..range.hi).collect()
}

/// Every non-empty range within the first six bytes.
fn small_ranges() -> Vec<ByteRange> {
    (0..6)
        .flat_map(|lo| (lo + 1..=6).map(move |hi| range(lo, hi)))
        .collect()
}

// Property: for all non-empty a and b, a.contains(b) == (bytes(b) ⊆ bytes(a)).
#[test]
fn contains_matches_the_byte_sets_on_small_ranges() {
    for a in small_ranges() {
        for b in small_ranges() {
            assert_eq!(a.contains(b), bytes(b).is_subset(&bytes(a)), "{a:?} {b:?}");
        }
    }
}

// Property: for all non-empty a and b, a.overlaps(b) == (bytes(a) ∩ bytes(b) is not empty).
#[test]
fn overlaps_matches_the_byte_sets_on_small_ranges() {
    for a in small_ranges() {
        for b in small_ranges() {
            let shared = !bytes(a).is_disjoint(&bytes(b));
            assert_eq!(a.overlaps(b), shared, "{a:?} {b:?}");
        }
    }
}

// Property: for all non-empty a and b, a.overlaps_or_adjacent(b) holds when they share a byte or one
// ends where the other starts.
#[test]
fn overlaps_or_adjacent_adds_touching_ranges_on_small_ranges() {
    for a in small_ranges() {
        for b in small_ranges() {
            let touching = !bytes(a).is_disjoint(&bytes(b)) || a.hi == b.lo || b.hi == a.lo;
            assert_eq!(a.overlaps_or_adjacent(b), touching, "{a:?} {b:?}");
        }
    }
}

// The call `person.greet(count(x))` of these tests: the receiver is bytes 0 to 6, `greet` 7 to 12,
// the argument `count(x)` 13 to 21, and `x` inside it 19 to 20.
const CALL: ByteRange = ByteRange { lo: 0, hi: 22 };
const RECEIVER: ByteRange = ByteRange { lo: 0, hi: 6 };
const ARGUMENT: ByteRange = ByteRange { lo: 13, hi: 21 };

/// The call's operands, as the driver passes them for `caret`: an operand is the caret only when its
/// bytes match, as they do when the two spans share an expansion context.
fn operands(caret: ByteRange) -> [Operand; 2] {
    [RECEIVER, ARGUMENT].map(|range| Operand {
        range,
        is_caret: range == caret,
    })
}

// Property: a caret is about a call when it covers the call, its method name, or one whole operand.
#[test]
fn a_caret_on_the_call_its_name_or_a_whole_operand_is_about_it() {
    for caret in [CALL, range(7, 12), ARGUMENT, RECEIVER] {
        assert!(is_about_call(caret, CALL, operands(caret)), "{caret:?}");
    }
}

// Concern: an error nested in an argument (`person.greet(count(Rc::new(1)))`) would be re-anchored
// onto the call and hidden.
// Handled: a caret strictly inside an operand is not about the call.
#[test]
fn a_caret_strictly_inside_an_operand_is_not_about_the_call() {
    let caret = range(19, 20);
    assert!(!is_about_call(caret, CALL, operands(caret)));
}

// Concern: an error beside the call (an `Rc` passed to a `T: Send` function on the line before)
// would be re-anchored onto it.
// Handled: a caret that does not overlap the call is not about it, even where it touches the call.
#[test]
fn a_caret_beside_the_call_is_not_about_it() {
    for caret in [range(30, 35), range(22, 25)] {
        assert!(!is_about_call(caret, CALL, operands(caret)), "{caret:?}");
    }
}

// Concern: a macro-expanded operand can have the caret's bytes in another expansion context.
// Handled: the caller says whether the spans are equal, and bytes alone do not make the caret whole.
#[test]
fn an_operand_with_the_carets_bytes_in_another_context_holds_the_caret() {
    let other_context = [Operand {
        range: ARGUMENT,
        is_caret: false,
    }];
    assert!(!is_about_call(ARGUMENT, CALL, other_context));
}

// The impl of these tests spans bytes 10 to 50, with one method body at 30 to 40.
const IMPL: ByteRange = ByteRange { lo: 10, hi: 50 };
const BODIES: [ByteRange; 1] = [ByteRange { lo: 30, hi: 40 }];

fn place(caret: ByteRange) -> CaretPlace {
    caret_place(caret, IMPL, &BODIES)
}

// Property: every caret is outside the impl, in a method body, or on the impl outside every body,
// and a caret that crosses a boundary belongs to the larger region.
#[test]
fn a_caret_is_placed_by_the_region_that_holds_it_whole() {
    assert_eq!(place(range(0, 5)), CaretPlace::OutsideImpl);
    assert_eq!(place(range(5, 15)), CaretPlace::OutsideImpl);
    assert_eq!(place(range(12, 20)), CaretPlace::ImplHeader);
    assert_eq!(place(range(25, 35)), CaretPlace::ImplHeader);
    assert_eq!(place(range(30, 40)), CaretPlace::MethodBody);
    assert_eq!(place(range(32, 33)), CaretPlace::MethodBody);
}
