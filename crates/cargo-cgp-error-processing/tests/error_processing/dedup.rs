//! The cross-diagnostic de-duplication ledger, over hand-built signatures.

use cargo_cgp_error_processing::DedupLedger;

/// A `text_signature` thunk that must not be called: for the resolved path, whose key is the
/// recovered cause.
fn no_text() -> String {
    panic!("the text signature must not be computed when a cause signature is present");
}

// Property: for all cause signatures s, the second record of s reports a re-report, and the text
//   signature is never computed when s is present (`DedupLedger::check_and_record`'s doc).
#[test]
fn a_resolved_failure_is_suppressed_on_its_second_report() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, &[]));
    // The same cause re-reported at another site (different rendered text is irrelevant).
    assert!(ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, &[]));
}

// Property: for all distinct cause signatures, each first record is reported
//   (`DedupLedger::check_and_record`'s doc).
#[test]
fn distinct_causes_are_each_reported() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("App\u{1f}CanGreet\u{1f}missing name"), no_text, &[]));
    assert!(!ledger.check_and_record(Some("App\u{1f}CanRun\u{1f}missing name"), no_text, &[]));
}

// Property: a diagnostic with no cause signature is keyed by its text signature, so an equal text
//   is a re-report and a different one is not (`DedupLedger::check_and_record`'s doc).
#[test]
fn a_declined_rewrite_is_keyed_by_its_rendered_text() {
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(None, || "some rewritten text".to_owned(), &[]));
    assert!(ledger.check_and_record(None, || "some rewritten text".to_owned(), &[]));
    assert!(!ledger.check_and_record(None, || "different text".to_owned(), &[]));
}

// Property: the cause and text key spaces are disjoint, so a text equal to a recorded cause
//   signature is not a re-report.
#[test]
fn a_cause_key_and_a_text_key_never_collide() {
    // The two key spaces are prefixed apart, so a text that happens to equal a cause signature
    // still gets its own entry.
    let mut ledger = DedupLedger::new();
    assert!(!ledger.check_and_record(Some("same"), no_text, &[]));
    assert!(!ledger.check_and_record(None, || "same".to_owned(), &[]));
}

// Property: a declined report whose coded header a resolved report recorded is a re-report
//   (`DedupLedger::check_and_record`'s doc).
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

// Property: a header without a `[CGP-E0` prefix is never a key, so a matching rustc header
//   suppresses nothing (`DedupLedger::check_and_record`'s doc).
#[test]
fn a_kept_rustc_header_is_never_a_dedup_key() {
    let mut ledger = DedupLedger::new();
    let rustc_header = "the trait bound `f64: Eq` is not satisfied";
    assert!(!ledger.check_and_record(Some("cause-a"), no_text, &[rustc_header]));
    // A different failure whose kept header happens to match is still reported, resolved or
    // declined: only a declined report consults the headers others recorded.
    assert!(!ledger.check_and_record(Some("cause-b"), no_text, &[rustc_header]));
    assert!(!ledger.check_and_record(None, || "another body".to_owned(), &[rustc_header]));
}

// Property: the header key works one way, so a resolved report is kept even when a declined report
//   recorded its header first (`DedupLedger::check_and_record`'s doc).
#[test]
fn a_declined_fallback_arriving_first_never_hides_the_resolved_tree() {
    let mut ledger = DedupLedger::new();
    let header = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    // The declined fallback is shown: nothing better has arrived yet.
    assert!(!ledger.check_and_record(None, || "fallback body".to_owned(), &[header]));
    // The resolved tree of the same failure must still be shown, not suppressed by the header.
    assert!(!ledger.check_and_record(Some("cause"), no_text, &[header]));
}

// Property: a resolved report never consults the recorded headers, so two resolved reports with
//   different causes are both kept (`DedupLedger::check_and_record`'s doc).
#[test]
fn two_resolved_reports_with_different_causes_are_both_kept() {
    // One consumer on one context, reached at two sites that recover different cause sets: the
    // shared header must not hide the second set (the flush coalesces what they share).
    let mut ledger = DedupLedger::new();
    let header = "[CGP-E001] the consumer trait `CanGreet` is not implemented for context `App`";
    assert!(!ledger.check_and_record(Some("cause-a"), no_text, &[header]));
    assert!(!ledger.check_and_record(Some("cause-a+b"), no_text, &[header]));
}

// Property: every header a resolved report records is a key for later declined reports, its
//   provider form included (`DedupLedger::check_and_record`'s doc).
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
