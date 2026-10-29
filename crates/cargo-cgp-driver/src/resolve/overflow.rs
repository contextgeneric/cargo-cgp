//! How a wiring overflow never resolves: a cycle in the wiring, or a lookup that grows.

use cargo_cgp_error_processing::WiringOverflow;
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _};
use rustc_span::Span;

use crate::resolve::anchor::check_entry_obligation;
use crate::resolve::call_site::{CallForm, method_calls_at, receiver_context, seed_from_call};
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::query::{CallTrait, is_context_adt, traits_with_method};
use crate::resolve::walk::walk_overflow;

/// What walking the obligation at an `E0275` overflow's caret shows about it.
pub enum OverflowWalk {
    /// No check entry or consumer-method call sits at the span, so the walk cannot say why the
    /// lookup overflowed.
    NoEntry,
    /// The walk ran to its leaves: the overflow follows from something other than the wiring, such
    /// as a type the compiler could not resolve.
    NotCut,
    /// The walk met an obligation again, or reached its depth cap, as the reason says.
    Cut(WiringOverflow),
}

/// Walk the consumer obligation at `primary_span`, the overflow's caret, and report how its lookup
/// never resolves (see [`OverflowWalk`]). The obligation is a `check_components!` entry's, or else
/// that of the consumer-method call at the caret whose consumer trait `requirement` names, as the
/// overflow's message gives it (`Person: CanGreet`).
pub fn overflow_walk(
    cx: ResolveCtx<'_, '_>,
    primary_span: Span,
    requirement: &str,
) -> OverflowWalk {
    let tcx = cx.tcx;
    let obligation = check_entry_obligation(tcx, primary_span)
        .or_else(|| call_obligation(tcx, primary_span, requirement));
    let Some(obligation) = obligation else {
        return OverflowWalk::NoEntry;
    };
    match walk_overflow(cx, obligation) {
        Some(reason) => OverflowWalk::Cut(reason),
        None => OverflowWalk::NotCut,
    }
}

/// The consumer obligation of the method call at `primary_span`, when the call's consumer trait is
/// the one `requirement` (`Context: Trait<…>`) names. The call is recovered as the call-site anchor
/// recovers a failing call: a context read off the receiver, and the consumer's parameters from the
/// call's written argument types.
fn call_obligation<'tcx>(
    tcx: TyCtxt<'tcx>,
    primary_span: Span,
    requirement: &str,
) -> Option<ty::PolyTraitClause<'tcx>> {
    let (_, trait_ref) = requirement.split_once(": ")?;
    let path = trait_ref.split('<').next()?;
    let name = path.rsplit("::").next()?;
    for call in method_calls_at(tcx, &[primary_span]) {
        let Some(context) = receiver_context(tcx, call.receiver) else {
            continue;
        };
        if !is_context_adt(tcx, context) || context.has_param() {
            continue;
        }
        for (trait_did, method_did, kind) in traits_with_method(tcx, call.method) {
            if kind != CallTrait::Consumer || tcx.item_name(trait_did).as_str() != name {
                continue;
            }
            if let Some(top) = seed_from_call(
                tcx,
                context,
                trait_did,
                method_did,
                call.args,
                CallForm::Method,
            ) {
                return Some(top);
            }
        }
    }
    None
}
