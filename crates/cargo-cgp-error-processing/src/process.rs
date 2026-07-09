//! The error-processing entrypoint.

use cargo_metadata::diagnostic::Diagnostic;

use crate::diagnostic::CgpDiagnostic;
use crate::preprocess::preprocess;

/// Transform the diagnostics rustc produced into CGP diagnostics.
///
/// Each raw diagnostic is wrapped into a [`CgpDiagnostic`] and run through the
/// [preprocessing pipeline](crate::preprocess::preprocess), which cleans up and resugars
/// it on its own. The input is taken by value so wrapping is a move, not a clone.
///
/// # This is only the preprocessing stage — the aggregation stage is still missing
///
/// Preprocessing is deliberately per-diagnostic: it maps each diagnostic independently,
/// so the output has exactly one entry per input. That is correct *for this stage* and
/// is why the body is a `map`. It is **not** the whole processor. The stage still to come
/// is aggregation: one CGP mistake cascades into many diagnostics, and the processor must
/// detect the repetition, lift the single root cause to the top, and drop or summarize the
/// echoes — a transform that returns fewer diagnostics than it received and that can only
/// be decided by looking at the *whole set*.
///
/// So the shape to grow into is two phases: this per-diagnostic preprocessing map,
/// followed by an aggregation phase that ingests all the preprocessed diagnostics into a
/// queryable store and synthesizes the output from a view across them. Do not fold
/// aggregation into the per-diagnostic map below — a `map` can never deduplicate a
/// cascade, because each step is blind to the others. See
/// `docs/implementation/error-processing.md`.
///
/// # Input type note
///
/// The input is [`cargo_metadata::Diagnostic`], the public, deserializable mirror of
/// rustc's JSON diagnostic shape, because the diagnostics must serialize to fixture files
/// and this function must be callable without the compiler. The alternative — kept here
/// in case the `cargo_metadata` route proves insufficient — is the compiler's own
/// in-memory `rustc_errors::DiagInner`, read live inside the driver through a custom
/// `Emitter`. `DiagInner` carries richer, un-rendered structure (interned messages,
/// unresolved `MultiSpan`s, the raw arg map) that a future analysis might need, but it is
/// `rustc_private`, is not serializable, and has no rendered form — so adopting it would
/// move this function into the driver crate and cost it its standalone testability. We
/// take the `cargo_metadata` route first for that reason and reconsider `DiagInner` only
/// if it cannot carry enough.
pub fn process_cgp_errors(rust_errors: Vec<Diagnostic>) -> Vec<CgpDiagnostic> {
    rust_errors
        .into_iter()
        .map(CgpDiagnostic::wrap)
        .map(preprocess)
        .collect()
}
