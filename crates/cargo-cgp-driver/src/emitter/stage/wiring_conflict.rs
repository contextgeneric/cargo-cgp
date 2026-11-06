//! The stage for a duplicate-key wiring conflict.

use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::DiagInner;
use rustc_errors::codes::E0119;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::config::{DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT};
use crate::detect::{self, ConflictAction, ConflictTrait};
use crate::emitter::edit::main_message_text;
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};

/// Stage 6: handle the `E0119` a duplicate `delegate_components!` or `cgp_namespace!` entry
/// produces as one logical error. The redundant `IsProviderFor` half of the pair is dropped, and
/// the `DelegateComponent` half is reworded to name the colliding keys, keeping rustc's two carets.
/// The message text routes the shape, and the typed classifier verifies that genuine CGP impls sit
/// at the caret before either fires.
pub fn wiring_conflict(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if diag.code != Some(E0119) {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(message), Some(primary_span)) =
        (tcx, main_message_text(diag), diag.span.primary_span())
    else {
        return Verdict::Continue;
    };
    // `IsProviderFor` is checked first: a `DelegateComponent` conflict names only
    // `DelegateComponent`, while its companion names `IsProviderFor`. A message naming neither can
    // still be a `cgp_namespace!` conflict on the user's own namespace trait, which the classifier
    // recognizes by the impls at the carets.
    let variant = if message.contains(IS_PROVIDER_FOR_TRAIT) {
        ConflictTrait::IsProviderFor
    } else if message.contains(DELEGATE_COMPONENT_TRAIT) {
        ConflictTrait::Delegate
    } else {
        ConflictTrait::Other
    };
    let label_spans: Vec<Span> = diag
        .span
        .span_labels()
        .into_iter()
        .map(|label| label.span)
        .collect();
    match detect::classify_wiring_conflict(tcx, variant, primary_span, &label_spans) {
        Some(ConflictAction::Suppress) => Verdict::Drop,
        Some(ConflictAction::Rewrite(conflict)) => Verdict::Reshape(Reshape::new(
            ClassPlan::wiring_conflict(&conflict),
            SpanPolicy::Keep,
        )),
        None => Verdict::Continue,
    }
}
