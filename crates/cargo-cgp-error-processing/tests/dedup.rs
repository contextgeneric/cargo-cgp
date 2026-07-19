//! The cross-diagnostic de-duplication ledger, over hand-built signatures.

use cargo_cgp_error_processing::DedupLedger;

/// A `text_signature` thunk that must not be called — for the resolved path, whose key is the
/// recovered cause.
fn no_text() -> String {
    panic!("the text signature must not be computed when a cause signature is present");
}

#[test]
fn a_resolved_failure_is_suppressed_on_its_second_report() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, None));
    // The same cause re-reported at another site (different rendered text is irrelevant).
    assert!(ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, None));
}

#[test]
fn distinct_causes_are_each_reported() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, None));
    assert!(!ledger.check_and_record(Some("App\u{1f}CanRun\u{1f}missing name"), no_text, None));
}

#[test]
fn a_declined_rewrite_is_keyed_by_its_rendered_text() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(None, || "some rewritten text".to_owned(), None));
    assert!(ledger.check_and_record(None, || "some rewritten text".to_owned(), None));
    assert!(!ledger.check_and_record(None, || "different text".to_owned(), None));
}

#[test]
fn a_cause_key_and_a_text_key_never_collide() {
    // The two key spaces are prefixed apart, so a text that happens to equal a cause signature
    // still gets its own entry.
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("same"), no_text, None));
    assert!(!ledger.check_and_record(None, || "same".to_owned(), None));
}

#[test]
fn a_coded_header_collapses_a_declined_fallback_into_the_resolved_occurrence() {
    let mut ledger = DedupLedger::new();
    let header = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    // The resolved occurrence records both its cause and its coded header.
    assert!(!ledger.check_and_record(Some("cause"), no_text, Some(header)));
    // A declined-but-rewritten re-report has a different body (text key) but the same coded
    // header, so it is recognized as a re-report.
    assert!(ledger.check_and_record(None, || "fallback body".to_owned(), Some(header)));
}

#[test]
fn a_kept_rustc_header_is_never_a_dedup_key() {
    let mut ledger = DedupLedger::new();
    let rustc_header = "the trait bound `f64: Eq` is not satisfied";
    assert!(!ledger.check_and_record(Some("cause-a"), no_text, Some(rustc_header)));
    // A different failure whose kept header happens to match is still reported.
    assert!(!ledger.check_and_record(Some("cause-b"), no_text, Some(rustc_header)));
}
