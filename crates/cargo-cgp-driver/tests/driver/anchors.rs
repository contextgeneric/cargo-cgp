//! Tests for the anchor list's gates and acceptance, the decisions it makes before and after an
//! anchor runs: a diagnostic's shape reaches only the anchors that can read it.

use cargo_cgp_driver::resolve::{ANCHORS, AnchorInput};
use cargo_cgp_error_processing::{Causes, ChainNode, Leaf, Resolved, ResolvedClass};
use rustc_errors::ErrCode;
use rustc_errors::codes::{E0271, E0277, E0308, E0599};
use rustc_span::DUMMY_SP;

/// The position of the data-call anchor in the list, the eighth tried.
const DATA_CALL: usize = 7;

fn input(code: ErrCode, data_call_only: bool) -> AnchorInput<'static> {
    AnchorInput {
        code: Some(code),
        primary_span: DUMMY_SP,
        primary_spans: &[],
        spans: Vec::new(),
        data_call_only,
    }
}

/// The positions in the list whose gate lets `input` through.
fn reached(input: &AnchorInput<'_>) -> Vec<usize> {
    ANCHORS
        .iter()
        .enumerate()
        .filter(|(_, anchor)| (anchor.gate)(input))
        .map(|(position, _)| position)
        .collect()
}

// Property: for all data-call-only diagnostics, the only entry reached is the data-call anchor.
#[test]
fn a_data_call_only_shape_reaches_the_data_call_anchor_alone() {
    assert_eq!(reached(&input(E0599, true)), [DATA_CALL]);
    assert_eq!(reached(&input(E0308, true)), [DATA_CALL]);
}

// Property: an unmet obligation other than `E0277` reaches the first nine entries, and the last
// three read a trait used as a bound, so they stay shut.
#[test]
fn the_bound_anchors_stay_shut_for_a_projection_and_a_method_call() {
    let first_nine: Vec<usize> = (0..9).collect();
    assert_eq!(reached(&input(E0271, false)), first_nine);
    assert_eq!(reached(&input(E0599, false)), first_nine);
}

// Property: an `E0277` reaches every entry.
#[test]
fn a_trait_bound_failure_reaches_every_anchor() {
    let every: Vec<usize> = (0..ANCHORS.len()).collect();
    assert_eq!(reached(&input(E0277, false)), every);
}

fn resolved_with(leaf: Leaf) -> Resolved {
    Resolved {
        context: "Person".to_owned(),
        consumers: vec!["BuildField".to_owned()],
        class: ResolvedClass::DataOperation,
        subject_is_context: false,
        causes: Causes::from_sub_chains([(leaf.clone(), vec![ChainNode::Leaf(leaf)])]),
    }
}

fn data_state() -> Leaf {
    Leaf::FieldAlreadySet {
        name: "name".to_owned(),
        owner: "Person".to_owned(),
    }
}

fn plain_bound() -> Leaf {
    Leaf::Bound {
        summary: "u8: Clone".to_owned(),
    }
}

// Property: an `E0308` is explained only by a field or variant state; every other shape the anchor
// reads is explained by whatever it resolved.
// Concern: a mismatched argument is usually a plain type error, which a resolution must not replace.
// Handled: the entry's `accept` reads each cause's leaf.
#[test]
fn the_data_call_anchor_accepts_a_mismatch_only_for_a_data_state() {
    let accept = ANCHORS[DATA_CALL].accept;
    assert!(accept(&input(E0308, true), &resolved_with(data_state())));
    assert!(!accept(&input(E0308, true), &resolved_with(plain_bound())));
    assert!(accept(&input(E0599, true), &resolved_with(plain_bound())));
    assert!(accept(&input(E0277, false), &resolved_with(plain_bound())));
}
