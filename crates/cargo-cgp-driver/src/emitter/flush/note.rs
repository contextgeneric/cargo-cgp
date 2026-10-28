//! Rendering a resolution's deferred `root cause:` note into its diagnostic.

use cargo_cgp_error_processing::{AmbiguousNames, Origin, PendingNote, postprocess_message};
use rustc_errors::{DiagInner, Sublevel};

use crate::emitter::edit::subdiag;
use crate::emitter::flush::ChainNodeSet;

/// Render a deferred [`PendingNote`] against the `seen` set shared by this flush and add it to
/// `diag`, ahead of its `help`s, so each fix follows the cause it repairs. The note is
/// post-processed on its own, since the rest of the diagnostic was already processed when it
/// arrived and re-running the chain over it could compound.
pub fn insert_note(
    diag: &mut DiagInner,
    note: &PendingNote,
    seen: &mut ChainNodeSet,
    ambiguous: &AmbiguousNames,
) {
    // A resolver-built note never carries the `` `HasField<…>` is not implemented `` clause the
    // missing-field reword keys on (see
    // cgp-knowledge-base/cargo-cgp/implementation/error-processing.md), so that branch has
    // nothing to match and the flag it needs is moot here.
    const HAS_FIELD_IMPLS: bool = false;
    let notes: Vec<_> = note
        .render(seen)
        .into_iter()
        .map(|text| {
            let text = postprocess_message(&text, HAS_FIELD_IMPLS, Origin::Rewritten, ambiguous)
                .unwrap_or(text);
            subdiag(Sublevel::Note, text)
        })
        .collect();
    let first_help = diag
        .children
        .iter()
        .position(|child| child.level == Sublevel::Help)
        .unwrap_or(diag.children.len());
    diag.children.splice(first_help..first_help, notes);
}
