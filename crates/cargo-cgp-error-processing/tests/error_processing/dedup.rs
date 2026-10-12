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
    assert!(!ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, &[]));
    // The same cause re-reported at another site (different rendered text is irrelevant).
    assert!(ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, &[]));
}

#[test]
fn distinct_causes_are_each_reported() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, &[]));
    assert!(!ledger.check_and_record(Some("App\u{1f}CanRun\u{1f}missing name"), no_text, &[]));
}

#[test]
fn a_declined_rewrite_is_keyed_by_its_rendered_text() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(None, || "some rewritten text".to_owned(), &[]));
    assert!(ledger.check_and_record(None, || "some rewritten text".to_owned(), &[]));
    assert!(!ledger.check_and_record(None, || "different text".to_owned(), &[]));
}

#[test]
fn a_cause_key_and_a_text_key_never_collide() {
    // The two key spaces are prefixed apart, so a text that happens to equal a cause signature
    // still gets its own entry.
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("same"), no_text, &[]));
    assert!(!ledger.check_and_record(None, || "same".to_owned(), &[]));
}

#[test]
fn a_coded_header_collapses_a_declined_fallback_into_the_resolved_occurrence() {
    let mut ledger = DedupLedger::new();
    let header = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    // The resolved occurrence records both its cause and its coded header.
    assert!(!ledger.check_and_record(Some("cause"), no_text, &[header]));
    // A declined-but-rewritten re-report has a different body (text key) but the same coded
    // header, so it is recognized as a re-report.
    assert!(ledger.check_and_record(None, || "fallback body".to_owned(), &[header]));
}

#[test]
fn a_kept_rustc_header_is_never_a_dedup_key() {
    let mut ledger = DedupLedger::new();
    let rustc_header = "the trait bound `f64: Eq` is not satisfied";
    assert!(!ledger.check_and_record(Some("cause-a"), no_text, &[rustc_header]));
    // A different failure whose kept header happens to match is still reported.
    assert!(!ledger.check_and_record(Some("cause-b"), no_text, &[rustc_header]));
}

#[test]
fn a_declined_fallback_arriving_first_never_hides_the_resolved_tree() {
    let mut ledger = DedupLedger::new();
    let header = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    // The declined fallback is shown: nothing better has arrived yet.
    assert!(!ledger.check_and_record(None, || "fallback body".to_owned(), &[header]));
    // The resolved tree of the same failure must still be shown, not suppressed by the header.
    assert!(!ledger.check_and_record(Some("cause"), no_text, &[header]));
}

#[test]
fn two_resolved_reports_with_different_causes_are_both_kept() {
    // One consumer on one context, reached at two sites that recover different cause sets: the
    // shared header must not hide the second set (the flush coalesces what they share).
    let mut ledger = DedupLedger::new();
    let header = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    assert!(!ledger.check_and_record(Some("cause-a"), no_text, &[header]));
    assert!(!ledger.check_and_record(Some("cause-a+b"), no_text, &[header]));
}

#[test]
fn a_second_header_matches_a_fallback_worded_another_way() {
    // A resolved report classed by its consumer also records the provider form the text rewrite
    // gives rustc's own headline, so a declined fallback worded that way is a re-report too.
    let mut ledger = DedupLedger::new();
    let class = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    let provider_form = "[CGP-E002] the provider trait `Greeter` with context `App` is not \
                         implemented for provider `GreetHello`";
    assert!(!ledger.check_and_record(Some("cause"), no_text, &[class, provider_form]));
    assert!(ledger.check_and_record(None, || "fallback body".to_owned(), &[provider_form]));
}
