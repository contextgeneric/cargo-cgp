//! Containing a panic in the emitter's own code, and the panic hook that makes it quiet.
//!
//! The emitter re-runs compiler code from inside `emit_diagnostic` and renders every buffered
//! diagnostic from its `Drop`, so a bug in either would otherwise lose the crate's diagnostics or
//! abort the process. [`contain`] runs such work under `catch_unwind`, and the hook
//! [`install_panic_hooks`] sets up keeps a contained panic from printing an ICE report, since the
//! compilation goes on. Every other panic is a real crash: the hook first has the emitter print
//! the diagnostics it buffered ([`flush_registered_output`]), so they appear before the crash
//! report as they do in plain rustc, and then hands over to rustc's ICE hook, which names
//! cargo-cgp's issue tracker. See `cgp-knowledge-base/cargo-cgp/implementation/driver.md`.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::panic::{self, AssertUnwindSafe, PanicHookInfo};

use rustc_data_structures::FatalErrorMarker;
use rustc_errors::DiagCtxtHandle;

use crate::emitter::output::flush_registered_output;
use crate::version::TOOL_VERSION;

thread_local! {
    /// Whether this thread is inside [`contain`], so the panic hook stays quiet.
    static CONTAINING: Cell<bool> = const { Cell::new(false) };
    /// The description the panic hook recorded for the panic [`contain`] is catching.
    static CAUGHT: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Run `f`, containing a panic in it: `Err` carries the panic's message and location. A
/// `FatalErrorMarker` unwind is rustc aborting on a fatal error, not a panic of ours, so it
/// continues unwinding.
pub fn contain<R>(f: impl FnOnce() -> R) -> Result<R, String> {
    let outer = CONTAINING.with(|containing| containing.replace(true));
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    CONTAINING.with(|containing| containing.set(outer));

    result.map_err(|payload| {
        if payload.is::<FatalErrorMarker>() {
            panic::resume_unwind(payload);
        }
        CAUGHT
            .with(|caught| caught.borrow_mut().take())
            .unwrap_or_else(|| payload_message(payload.as_ref()).to_owned())
    })
}

/// Install rustc's ICE hook with cargo-cgp's bug-report URL, then wrap it so a contained panic is
/// recorded instead of reported, and an uncontained one prints the buffered diagnostics first.
/// Call once, before the compiler runs.
pub fn install_panic_hooks(bug_report_url: &'static str) {
    rustc_driver::install_ice_hook(bug_report_url, add_version_note);

    let ice_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if CONTAINING.try_with(Cell::get).unwrap_or(false) {
            let _ = CAUGHT.try_with(|caught| *caught.borrow_mut() = Some(describe(info)));
            return;
        }
        flush_registered_output();
        ice_hook(info);
    }));
}

/// The extra line the ICE report carries, naming the driver version that crashed.
fn add_version_note(dcx: DiagCtxtHandle<'_>) {
    dcx.note(format!("cargo-cgp-driver version: {TOOL_VERSION}"));
}

/// A panic's message and where it was raised.
fn describe(info: &PanicHookInfo<'_>) -> String {
    let message = payload_message(info.payload());
    match info.location() {
        Some(location) => format!(
            "{message} (at {}:{}:{})",
            location.file(),
            location.line(),
            location.column()
        ),
        None => message.to_owned(),
    }
}

/// The text of a panic payload, for the two payload types `panic!` produces.
fn payload_message(payload: &(dyn Any + Send)) -> &str {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else {
        "a panic with a non-string payload"
    }
}
