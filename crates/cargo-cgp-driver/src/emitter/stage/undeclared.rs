//! The stages for a dependency a CGP body uses without declaring it: an undeclared trait, and an
//! inner provider used without `#[use_provider]`.

use cargo_cgp_error_processing::{
    ClassPlan, is_method_bounds_text, is_unbounded_type_param_item_text,
};
use rustc_errors::DiagInner;
use rustc_errors::codes::E0599;
use rustc_middle::ty::TyCtxt;

use crate::emitter::edit::{diag_mentions, diagnostic_spans, main_message_text};
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};
use crate::resolve;

/// Stage 10: reword the vague `E0599` a trait called in a `#[cgp_fn]` or `#[cgp_impl]` body
/// without `#[uses(…)]` produces, since its method cannot resolve on the generated `__Context__`.
/// The reshape names the trait and carries the `#[uses(…)]` fix, keeping the caret on the call.
/// Gated to the two `E0599` shapes a method call and a `Self::method(…)` call produce: the
/// method-bounds message, or, for the second, whose main message is a Fluent one, its help.
/// [`resolve::detect_undeclared_trait`] confirms it structurally.
pub fn undeclared_trait(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if diag.code != Some(E0599)
        || !(main_message_text(diag).is_some_and(is_method_bounds_text)
            || diag_mentions(diag, is_unbounded_type_param_item_text))
    {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
        return Verdict::Continue;
    };
    let spans = diagnostic_spans(diag);
    match resolve::detect_undeclared_trait(tcx, primary_span, &spans) {
        Some(undeclared) => Verdict::Reshape(Reshape::new(
            ClassPlan::undeclared_trait(&undeclared),
            SpanPolicy::Primary,
        )),
        None => Verdict::Continue,
    }
}

/// Stage 11: reword the `E0599` a higher-order provider produces when it calls an inner provider
/// it never imported with `#[use_provider]`, which leaks `__Context__` and suggests the wrong
/// (consumer-trait) bound. Gated to the "no associated item for a type parameter" shape, reported
/// during the calling body's typeck, where the detector's queries are cached (unlike the
/// resolution-class `E0599` emitted mid-`clauses_of`); its main message is a Fluent one, so its help
/// is read. [`resolve::detect_missing_use_provider`] confirms it structurally.
pub fn missing_use_provider(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if diag.code != Some(E0599) || !diag_mentions(diag, is_unbounded_type_param_item_text) {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
        return Verdict::Continue;
    };
    match resolve::detect_missing_use_provider(tcx, primary_span) {
        Some(missing) => Verdict::Reshape(Reshape::new(
            ClassPlan::missing_use_provider(&missing),
            SpanPolicy::Primary,
        )),
        None => Verdict::Continue,
    }
}
