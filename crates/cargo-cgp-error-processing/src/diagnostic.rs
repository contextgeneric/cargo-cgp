//! The CGP diagnostic type — the output of the processing stage.

use cargo_metadata::diagnostic::Diagnostic;

/// One CGP diagnostic, a structural superset of a rustc diagnostic.
///
/// A `CgpDiagnostic` always carries the underlying rustc [`Diagnostic`], and will grow
/// optional CGP-specific fields — a classified error class, decoded type-level
/// encodings, the link from a root cause to the cascade it explains — as the processing
/// stage learns to recognize CGP errors. The superset shape lets one type serve both a
/// **passed-through** diagnostic (a non-CGP error, or a CGP error not yet handled, whose
/// extra fields are empty) and a **synthesized** CGP diagnostic (whose extra fields hold
/// what the analysis recovered), so rendering never has to special-case the two.
///
/// The base data is kept as structured `cargo_metadata::Diagnostic`, never a
/// pre-rendered string, so a later render stage can produce more than one form from it —
/// human-readable text today, the `--message-format=json` form later.
#[derive(Debug, Clone)]
pub struct CgpDiagnostic {
    /// The underlying rustc diagnostic this was built from.
    pub diagnostic: Diagnostic,
}

impl CgpDiagnostic {
    /// Wrap a rustc diagnostic as a pass-through CGP diagnostic — no CGP analysis
    /// applied. This is how a non-CGP error (or one the processor does not yet handle)
    /// reaches the output unchanged.
    pub fn passthrough(diagnostic: Diagnostic) -> Self {
        Self { diagnostic }
    }

    /// The human-rendered form rustc produced for this diagnostic, if any. The render
    /// stage prints this to reproduce rustc's own pretty output.
    pub fn rendered(&self) -> Option<&str> {
        self.diagnostic.rendered.as_deref()
    }
}
