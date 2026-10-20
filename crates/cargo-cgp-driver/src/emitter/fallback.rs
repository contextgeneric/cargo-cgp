//! What the emitter emits when its own transform or flush panics: the diagnostics rustc built.
//!
//! The emitter's two risky steps run through these two functions, which take the step as a
//! closure. The emitter passes its real transform and render; a test passes a closure that panics,
//! so the fallback is tested without a switch in the shipped code. See
//! `cgp-knowledge-base/cargo-cgp/implementation/driver.md`.

use cargo_cgp_error_processing::internal_error_note;
use rustc_errors::{DiagInner, Sublevel};

use crate::config::BUG_REPORT_URL;
use crate::emitter::containment::contain;
use crate::emitter::edit::subdiag;

/// Run `transform` on `diag`, and return `None` when it finishes, or, when it panics, the
/// diagnostic as rustc built it with a note naming the internal error, for the caller to buffer
/// in its place. `transform` receives the diagnostic and a copy of it as rustc built it.
pub fn transform_or_original(
    diag: DiagInner,
    transform: impl FnOnce(DiagInner, &DiagInner),
) -> Option<DiagInner> {
    let original = diag.clone();
    let detail = contain(|| transform(diag, &original)).err()?;
    let mut original = original;
    original.children.push(subdiag(
        Sublevel::Note,
        internal_error_note(&detail, BUG_REPORT_URL),
    ));
    Some(original)
}

/// The diagnostics to emit for `buffer`: what `render` makes of it, or, when `render` panics, each
/// entry as rustc built it through `untransformed`, which receives the note naming the internal
/// error.
pub fn render_or_untransformed<T>(
    buffer: Vec<T>,
    render: impl FnOnce(&[T]) -> Vec<DiagInner>,
    mut untransformed: impl FnMut(T, &str) -> DiagInner,
) -> Vec<DiagInner> {
    match contain(|| render(&buffer)) {
        Ok(rendered) => rendered,
        Err(detail) => {
            let note = internal_error_note(&detail, BUG_REPORT_URL);
            buffer
                .into_iter()
                .map(|entry| untransformed(entry, &note))
                .collect()
        }
    }
}
