//! The note on a diagnostic the driver shows untransformed after an internal error.
//!
//! The driver runs every transform, and the flush that renders the buffered diagnostics, so that a
//! panic in its own code falls back to the diagnostic rustc built instead of losing it. The reader
//! then sees rustc's wording where they may expect the tool's, so the fallback carries this note:
//! it says the message is rustc's own, what failed, and where to report the bug.

/// The note attached to a diagnostic shown as rustc reported it because the tool's transform of it
/// panicked. `detail` describes the panic (its message and location), and `report_url` is where the
/// bug is reported.
pub fn internal_error_note(detail: &str, report_url: &str) -> String {
    format!(
        "cargo-cgp could not transform this diagnostic because of an internal error, so it is \
         shown as the compiler reported it: {detail}\n\
         this is a bug in cargo-cgp; please report it at {report_url}"
    )
}
