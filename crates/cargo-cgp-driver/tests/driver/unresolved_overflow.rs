//! Tests for recognizing an overflow that follows from a wiring value that does not resolve.
//!
//! A wiring entry whose value names nothing leaves its context unable to use the component, and a
//! later use reports an `E0275` overflow on that requirement. The name-resolution pass records the
//! requirement without the component's parameters (`Person: CanGreet`), and the emitter drops an
//! overflow on it, with whatever parameters the component takes, as the entry's consequence
//! (`cgp-knowledge-base/cargo-cgp/implementation/driver.md`, "Reshaping an unresolved wiring
//! name").

use cargo_cgp_driver::emitter::is_unresolved_wiring_overflow;
use rustc_errors::codes::{E0275, E0277};
use rustc_errors::{DiagInner, ErrCode, Level};
use rustc_span::create_default_session_globals_then;

/// The requirements recorded for one unresolved value: the consumer trait for a call, and the
/// check form for a check.
fn recorded() -> Vec<String> {
    vec![
        "Person: CanGreet".to_owned(),
        "Person: CanUseComponent<GreeterComponent".to_owned(),
    ]
}

/// An `overflow evaluating the requirement` error on `requirement`, with `code`.
fn overflow(code: ErrCode, requirement: &str) -> DiagInner {
    let mut diag = DiagInner::new(
        Level::Error,
        format!("overflow evaluating the requirement `{requirement}`"),
    );
    diag.code = Some(code);
    diag
}

/// Whether the emitter drops an `E0275` on `requirement` given the recorded requirements.
fn drops(requirement: &str) -> bool {
    create_default_session_globals_then(|| {
        is_unresolved_wiring_overflow(&overflow(E0275, requirement), &recorded())
    })
}

// Property: for all recorded requirements r and parameter lists p (empty, a consumer trait's `<…>`,
//   or the rest of a check's `>` or `, …>`), an `E0275` on r followed by p is dropped.
#[test]
fn an_overflow_on_a_recorded_requirement_is_dropped() {
    assert!(drops("Person: CanGreet"), "the bare requirement");
    assert!(
        drops("Person: CanGreet<u32>"),
        "a consumer trait's arguments"
    );
    assert!(
        drops("Person: CanUseComponent<GreeterComponent>"),
        "a check of a component without parameters",
    );
    assert!(
        drops("Person: CanUseComponent<GreeterComponent, (u32,)>"),
        "a check of a component with parameters",
    );
}

// Concern: a requirement whose trait name only begins with a recorded one, or another context's,
//   could be dropped as if it were the entry's consequence.
// Handled: the recorded text must end where a parameter list begins, or at the end.
#[test]
fn an_overflow_on_another_requirement_is_kept() {
    assert!(!drops("Person: CanGreetAll"), "a longer trait name");
    assert!(!drops("PersonalApp: CanGreet"), "another context");
    assert!(!drops("App: CanGreet"), "a context no entry recorded");
}

// Property: only an `E0275` overflow is the entry's consequence; another code on the same text is
//   kept.
#[test]
fn another_code_is_kept() {
    create_default_session_globals_then(|| {
        assert!(
            !is_unresolved_wiring_overflow(&overflow(E0277, "Person: CanGreet"), &recorded()),
            "an `E0277` is not an overflow",
        );
    });
}

// Property: with nothing recorded, no overflow is dropped.
#[test]
fn nothing_recorded_drops_nothing() {
    create_default_session_globals_then(|| {
        assert!(
            !is_unresolved_wiring_overflow(&overflow(E0275, "Person: CanGreet"), &[]),
            "no entry left a requirement unprovable",
        );
    });
}
