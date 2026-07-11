//! Preprocessor: mark the primary header of a CGP-transformed diagnostic.
//!
//! When `cargo-cgp` has transformed a diagnostic, its primary header line is rewritten from
//! rustc's `error[E0277]:` into `CGP[E0277]:` — the level word replaced by `CGP`, the
//! original Rust code kept in place. This flags on the primary line that the tool reshaped
//! the diagnostic, while preserving the code so `rustc --explain E0277` still works and the
//! trailing `--explain` line is left untouched. The mark is distinct from the `[CGP0001]`
//! error codes on fully-rewritten messages: those name a *new* CGP class, this wraps the
//! *existing* Rust code (see `docs/error-code.md`).
//!
//! It runs last in the pipeline, so `has_cgp_error` already reflects every earlier
//! recognizer. It also flags a diagnostic on its own when the header carries the driver's
//! wiring-message rename — phrasing plain rustc never produces — so a wiring failure with no
//! resugared leaf is still recognized as a CGP diagnostic and gets the mark.

use crate::diagnostic::CgpDiagnostic;

/// Phrasing the driver's wiring-message rewrite produces and plain rustc never does. Any of
/// these in the diagnostic marks it as one `cargo-cgp` transformed, even when no other
/// preprocessor recognized it.
const WIRING_MARKERS: &[&str] = &[
    "the consumer trait bound `",
    "the provider trait bound `",
    "required for the provider `",
    "required for the context `",
];

/// Flag the diagnostic as CGP when its header is a driver wiring rename, then — for any CGP
/// diagnostic — rewrite the leading `error[CODE]:` header into `CGP[CODE]:`.
pub fn mark_cgp_header(mut diagnostic: CgpDiagnostic) -> CgpDiagnostic {
    if !diagnostic.has_cgp_error {
        let text = diagnostic
            .diagnostic
            .rendered
            .as_deref()
            .unwrap_or(&diagnostic.diagnostic.message);
        if WIRING_MARKERS.iter().any(|marker| text.contains(marker)) {
            diagnostic.has_cgp_error = true;
        }
    }

    if diagnostic.has_cgp_error
        && let Some(rendered) = diagnostic.diagnostic.rendered.take()
    {
        diagnostic.diagnostic.rendered = Some(mark_header(&rendered));
    }

    diagnostic
}

/// Rewrite a leading `error[` into `CGP[`, so `error[E0277]: …` becomes `CGP[E0277]: …`. Only
/// the coded form at the very start is touched: the code is kept, the later
/// `rustc --explain E0277` line is not a header and is left alone, and a header with no Rust
/// code (`error:`) has none to follow, so it is left as rustc wrote it.
fn mark_header(rendered: &str) -> String {
    match rendered.strip_prefix("error[") {
        Some(rest) => format!("CGP[{rest}"),
        None => rendered.to_owned(),
    }
}
