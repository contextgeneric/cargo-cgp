//! Asking the trait solver whether a predicate already holds.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{self, TyCtxt, TypingMode};
use rustc_trait_selection::traits::ObligationCause;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;

use crate::resolve::query::placeholders_to_infer;

/// Whether `pred` already holds: a dependency that is satisfied and so is not descended into.
pub fn holds<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyTraitClause<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// Whether an associated-type projection already holds: used to tell a matching field type from a
/// mismatched one (`<Rectangle as HasField<Symbol!("height")>>::Value == f64` holds when `height`
/// is `f64`, fails when it is `i32`).
pub fn holds_projection<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyProjectionClause<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// Whether `pred` may hold for *some* value of the call-site unknowns it carries: each
/// unknown-input placeholder becomes an inference variable, and the solver is asked whether any
/// impl could apply. A leaf that fails only for the placeholder but may hold for a real value
/// states nothing the programmer can act on.
pub fn may_hold_for_some_unknown<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyTraitClause<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let pred = placeholders_to_infer(&infcx, pred);
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_may_hold(&obligation)
}
