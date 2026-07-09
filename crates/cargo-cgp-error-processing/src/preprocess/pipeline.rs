//! The preprocessing pipeline.

use crate::diagnostic::CgpDiagnostic;
use crate::preprocess::{extract_missing_fields, resugar_symbol, strip_cgp_prefixes};

/// The preprocessors, applied in order. Each takes and returns one `CgpDiagnostic`, so the
/// output of one feeds the next. The list is the whole pipeline — add a preprocessor by
/// adding it here. Order matters: prefix stripping runs first so later stages match the
/// bare CGP names (`Symbol`, `Chars`, …), and `Symbol!` resugaring runs before the
/// field-message rewrite, which matches the resugared `HasField<Symbol!("…")>` form.
const PREPROCESSORS: &[fn(CgpDiagnostic) -> CgpDiagnostic] =
    &[strip_cgp_prefixes, resugar_symbol, extract_missing_fields];

/// Run the preprocessing pipeline on one diagnostic.
///
/// Preprocessing transforms a diagnostic on its own — cleaning up type names, resugaring
/// encodings — and never looks across diagnostics. Aggregation (deduplicating a cascade,
/// lifting a root cause above the errors it explains) is a separate, later stage that sees
/// the whole set; it must not be folded in here.
pub fn preprocess(diagnostic: CgpDiagnostic) -> CgpDiagnostic {
    PREPROCESSORS
        .iter()
        .fold(diagnostic, |diagnostic, preprocess| preprocess(diagnostic))
}
