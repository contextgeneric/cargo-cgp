//! The error-processing entrypoint.

use cargo_metadata::diagnostic::Diagnostic;

use crate::diagnostic::CgpDiagnostic;

/// Transform the diagnostics rustc produced into CGP diagnostics.
///
/// This is the stateless middle of the error pipeline: it takes the raw diagnostics
/// captured from a compilation and returns the set to present to the user. It is a pure
/// function over serializable data — no compiler, no filesystem, no global state — which
/// is what lets it be exercised by snapshot tests without running the tool (see the
/// crate's `tests/`).
///
/// # This is a placeholder — do not grow it into a per-error map
///
/// The current body is a scaffold: it treats every input as a non-CGP error and passes
/// each one through unchanged, reproducing rustc's own output through the new interface
/// so the pipeline can be wired end to end before any analysis exists. **The one-to-one
/// shape below is a stand-in, not the design.**
///
/// The real processor must return a *different, usually smaller* number of diagnostics
/// than it received: one CGP mistake cascades into many diagnostics, and the whole point
/// of this stage is to detect the repetition, lift the single root cause to the top, and
/// drop or summarize the echoes. That is impossible to decide one diagnostic at a time,
/// because whether a diagnostic is a root cause or an echo is a fact about the *whole
/// set*. So the real implementation must work in two phases — first **ingest** every
/// diagnostic into an internal, queryable store, then **query** that store to synthesize
/// the output — and must not be extended by adding rewrite branches inside the walk
/// below, which would permanently fix it as a naive `map` that can never deduplicate a
/// cascade. Replace the placeholder with the ingest-then-query core; do not flesh it out
/// in place. See `docs/implementation/error-processing.md`.
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
pub fn process_cgp_errors(rust_errors: &[Diagnostic]) -> Vec<CgpDiagnostic> {
    // PLACEHOLDER passthrough — see the warning above. Not the shape of the real stage.
    rust_errors
        .iter()
        .cloned()
        .map(CgpDiagnostic::passthrough)
        .collect()
}
