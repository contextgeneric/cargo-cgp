//! Tests for containing a panic in the emitter, and for the diagnostics it emits in place of what a
//! panicking transform or flush would have produced.
//!
//! Each fallback takes its step as a closure, so a closure that panics stands in for a bug in the
//! emitter, with no switch in the shipped code.

use std::panic;
use std::sync::Once;

use cargo_cgp_driver::config::BUG_REPORT_URL;
use cargo_cgp_driver::emitter::{
    contain, install_panic_hooks, render_or_untransformed, transform_or_original,
};
use rustc_data_structures::FatalErrorMarker;
use rustc_errors::{DiagInner, DiagMessage, Level};

/// Install the driver's panic hooks once for this test binary, as the driver does before it runs
/// the compiler, so a contained panic is recorded with its location.
fn hooks() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| install_panic_hooks(BUG_REPORT_URL));
}

fn diag(message: &str) -> DiagInner {
    DiagInner::new(Level::Error, message.to_owned())
}

/// The plain-string messages of a diagnostic: its main message, then each child's.
fn texts(diag: &DiagInner) -> Vec<String> {
    let strings = |messages: &[(DiagMessage, _)]| {
        messages
            .iter()
            .filter_map(|(message, _)| match message {
                DiagMessage::Str(text) => Some(text.to_string()),
                _ => None,
            })
            .collect::<String>()
    };
    std::iter::once(strings(&diag.messages))
        .chain(diag.children.iter().map(|child| strings(&child.messages)))
        .collect()
}

// Property: for all closures f that return without panicking, `contain(f) == Ok(f())`.
#[test]
fn a_finished_closure_returns_its_value() {
    hooks();
    assert_eq!(contain(|| 5), Ok(5));
}

// Property: for a closure that panics with a message, `contain` returns `Err` holding the message
//   followed by where the panic was raised.
#[test]
fn a_contained_panic_names_its_message_and_where_it_was_raised() {
    hooks();
    let detail = contain(|| -> u32 { panic!("emitter bug") }).unwrap_err();

    assert!(detail.starts_with("emitter bug (at "), "{detail}");
    assert!(detail.contains("containment.rs"), "{detail}");
}

// Property: a contained panic's `Err` carries the text of a `String` payload, the form a `panic!`
//   formatting a runtime value takes (literal arguments fold into a `&str`), and a fixed
//   description for a payload with no text.
#[test]
fn a_contained_panic_reads_a_formatted_or_a_non_string_payload() {
    hooks();
    let step = String::from("a step");
    let detail = contain(|| -> u32 { panic!("emitter bug in {step}") }).unwrap_err();
    assert!(detail.starts_with("emitter bug in a step (at "), "{detail}");

    let detail = contain(|| -> u32 { panic::panic_any(7_u32) }).unwrap_err();
    assert!(
        detail.starts_with("a panic with a non-string payload"),
        "{detail}"
    );
}

// Property: a `FatalErrorMarker` unwind, rustc ending a failing compilation, is not contained and
//   keeps unwinding.
#[test]
fn a_fatal_error_keeps_unwinding() {
    hooks();
    let unwound =
        panic::catch_unwind(|| contain(|| panic::resume_unwind(Box::new(FatalErrorMarker))))
            .unwrap_err();

    assert!(unwound.is::<FatalErrorMarker>());
}

// Property: after a nested `contain` returns, the outer one still contains its own panic and
//   reports where it was raised.
#[test]
fn a_nested_contain_leaves_the_outer_one_containing() {
    // The inner call must restore the outer one's state, or the outer panic would reach the ICE
    // hook and lose its location.
    hooks();
    let detail = contain(|| -> u32 {
        assert!(contain(|| -> u32 { panic!("inner") }).is_err());
        panic!("outer")
    })
    .unwrap_err();

    assert!(detail.starts_with("outer (at "), "{detail}");
}

// Property: when the transform returns, `transform_or_original` returns `None`, so the caller
//   buffers nothing in its place.
#[test]
fn a_transform_that_finishes_leaves_nothing_in_its_place() {
    hooks();
    assert!(transform_or_original(diag("rustc's message"), |_, _| {}).is_none());
}

// Property: the transform receives the diagnostic and a copy of it as rustc built it, equal in
//   their messages.
#[test]
fn the_transform_receives_the_diagnostic_as_rustc_built_it() {
    hooks();
    let fallback = transform_or_original(diag("rustc's message"), |diag, original| {
        assert_eq!(texts(&diag), texts(original));
    });

    assert!(fallback.is_none());
}

// Property: when the transform panics, even after editing its diagnostic, `transform_or_original`
//   returns the diagnostic as rustc built it with a note added that names the panic and the
//   bug-report URL.
#[test]
fn a_transform_that_panics_gives_back_rustcs_diagnostic_with_a_note() {
    hooks();
    let fallback = transform_or_original(diag("rustc's message"), |mut diag, _| {
        diag.messages.clear();
        panic!("transform bug")
    })
    .expect("a panicking transform gives the diagnostic back");

    let texts = texts(&fallback);
    assert_eq!(texts[0], "rustc's message");
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(texts[1].contains("transform bug"), "{texts:?}");
    assert!(texts[1].contains(BUG_REPORT_URL), "{texts:?}");
}

// Property: when the render returns, `render_or_untransformed` emits what it made, without
//   calling the fallback.
#[test]
fn a_render_that_finishes_is_what_is_emitted() {
    hooks();
    let emitted = render_or_untransformed(
        vec!["first", "second"],
        |buffer| vec![diag(&buffer.join(" and "))],
        |_, _| unreachable!("a render that finishes needs no fallback"),
    );

    assert_eq!(
        emitted
            .iter()
            .map(|d| texts(d)[0].clone())
            .collect::<Vec<_>>(),
        ["first and second"]
    );
}

// Property: when the render panics, `render_or_untransformed` emits every buffered entry through
//   `untransformed`, in buffer order, each given the note that names the panic and the
//   bug-report URL.
#[test]
fn a_render_that_panics_emits_every_entry_untransformed_in_order() {
    hooks();
    let emitted = render_or_untransformed(
        vec!["first", "second"],
        |_| panic!("render bug"),
        |entry, note| {
            assert!(note.contains("render bug"), "{note}");
            assert!(note.contains(BUG_REPORT_URL), "{note}");
            diag(entry)
        },
    );

    assert_eq!(
        emitted
            .iter()
            .map(|d| texts(d)[0].clone())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
}
