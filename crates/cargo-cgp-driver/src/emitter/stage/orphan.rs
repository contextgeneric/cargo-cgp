//! The stage for an orphan-rule namespace registration.

use cargo_cgp_error_processing::{ClassPlan, mentions_orphan_param_text};
use rustc_errors::DiagInner;
use rustc_errors::codes::{E0117, E0210};
use rustc_middle::ty::TyCtxt;

use crate::detect;
use crate::emitter::edit::main_message_text;
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};

/// Stage 7: reword the `E0210` (or its sibling `E0117`) of a crate registering wiring into a
/// foreign namespace keyed on a foreign key. The raw error names the machinery parameter and frames
/// a wiring decision as a bare coherence rule; the reshape names the namespace and the key, keeps
/// the caret on the offending macro alone (its "uncovered type parameter" label no longer applies),
/// and carries the ownership fix. The `E0210`'s text must name the machinery parameter
/// (`__Components__` or `__Table__`) before the impl scan runs; the rarer `E0117`, whose message
/// names none, is left to the typed classifier alone.
pub fn orphan_conflict(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if !matches!(diag.code, Some(E0210) | Some(E0117)) {
        return Verdict::Continue;
    }
    if diag.code == Some(E0210) && !main_message_text(diag).is_some_and(mentions_orphan_param_text)
    {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
        return Verdict::Continue;
    };
    match detect::classify_orphan_conflict(tcx, primary_span) {
        Some(conflict) => Verdict::Reshape(Reshape::new(
            ClassPlan::orphan_conflict(&conflict),
            SpanPolicy::Primary,
        )),
        None => Verdict::Continue,
    }
}
