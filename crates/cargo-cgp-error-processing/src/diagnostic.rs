//! The CGP diagnostic type — the output of the processing stage.

use cargo_metadata::diagnostic::Diagnostic;

/// One CGP diagnostic, a structural superset of a rustc diagnostic.
///
/// A `CgpDiagnostic` always carries the underlying rustc [`Diagnostic`], and grows
/// CGP-specific structure alongside it as the processing stage recognizes CGP errors. The
/// superset shape lets one type serve both a **passed-through** diagnostic (a non-CGP
/// error, whose extra fields stay at their defaults) and one the processor has understood
/// and rewritten. The base data is kept as structured `cargo_metadata::Diagnostic`, never
/// a pre-rendered string, so a later render stage can produce more than one form from it —
/// human-readable text today, the `--message-format=json` form later.
#[derive(Debug, Clone)]
pub struct CgpDiagnostic {
    /// The underlying rustc diagnostic, transformed in place as preprocessing runs.
    pub diagnostic: Diagnostic,
    /// Whether preprocessing recognized a CGP construct in this diagnostic — a CGP path
    /// prefix, a `Symbol!` spine, and so on. Defaults to `false`; a plain non-CGP
    /// diagnostic keeps it `false` and passes through untouched.
    pub has_cgp_error: bool,
}

impl CgpDiagnostic {
    /// Wrap a rustc diagnostic before preprocessing, with no CGP analysis applied yet.
    pub fn wrap(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic,
            has_cgp_error: false,
        }
    }

    /// The human-rendered form rustc produced for this diagnostic, if any. The render
    /// stage prints this to reproduce rustc's own pretty output.
    pub fn rendered(&self) -> Option<&str> {
        self.diagnostic.rendered.as_deref()
    }
}
