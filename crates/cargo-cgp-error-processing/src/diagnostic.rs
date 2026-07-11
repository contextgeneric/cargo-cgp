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
    /// Structured facts a preprocessor extracted from the diagnostic, on top of rewriting
    /// its text. Empty by default; a later aggregation stage will read these to group and
    /// reorder diagnostics without re-parsing their text.
    pub details: Vec<CgpDiagnosticDetail>,
}

/// A structured fact preprocessing recovered from a diagnostic.
///
/// A detail records *what* a preprocessor understood, independently of how it rewrote the
/// message, so later stages (and eventual JSON output) can act on the fact rather than the
/// prose. The variants grow as preprocessors learn to recognize more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgpDiagnosticDetail {
    /// A context is missing a single field a getter needs, while implementing `HasField`
    /// for other fields — the fix is to add the one field.
    MissingField { field_name: String, context: String },
    /// A context has no `HasField` impls at all behind a `HasField` requirement — the fix
    /// is to add `#[derive(HasField)]`, not to add fields one at a time.
    MissingDeriveHasField { field_name: String, context: String },
}

impl CgpDiagnosticDetail {
    /// The CGP error code for a single missing field ([`MissingField`](Self::MissingField)).
    pub const MISSING_FIELD_CODE: &'static str = "CGP0001";
    /// The CGP error code for a missing `#[derive(HasField)]`
    /// ([`MissingDeriveHasField`](Self::MissingDeriveHasField)).
    pub const MISSING_DERIVE_CODE: &'static str = "CGP0002";

    /// The CGP error code identifying this class of fully-rewritten error. Each code names
    /// one entry in `docs/error-code.md`, and a preprocessor tags the message it rewrites
    /// with the code in a `[CGPxxxx]` prefix — deliberately unlike rustc's `E0277`, so the
    /// two schemes never blur. Codes are attached only to *full* message rewrites like these;
    /// the cosmetic partial rewrites (prefix stripping, `Symbol!` resugaring, the driver's
    /// wiring-message renaming) carry none.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingField { .. } => Self::MISSING_FIELD_CODE,
            Self::MissingDeriveHasField { .. } => Self::MISSING_DERIVE_CODE,
        }
    }
}

impl CgpDiagnostic {
    /// Wrap a rustc diagnostic before preprocessing, with no CGP analysis applied yet.
    pub fn wrap(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic,
            has_cgp_error: false,
            details: Vec::new(),
        }
    }

    /// The human-rendered form rustc produced for this diagnostic, if any. The render
    /// stage prints this to reproduce rustc's own pretty output.
    pub fn rendered(&self) -> Option<&str> {
        self.diagnostic.rendered.as_deref()
    }
}
