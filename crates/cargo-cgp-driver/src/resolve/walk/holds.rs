//! Asking the trait solver whether a predicate already holds.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{self, TyCtxt, TypingMode};
use rustc_trait_selection::traits::ObligationCause;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;

/// Whether `pred` already holds — a dependency that is satisfied and so is not descended into.
pub(crate) fn holds<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyTraitPredicate<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// Whether an associated-type projection already holds — used to tell a matching field type from a
/// mismatched one (`<Rectangle as HasField<Symbol!("height")>>::Value == f64` holds when `height`
/// is `f64`, fails when it is `i32`).
pub(crate) fn holds_projection<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyProjectionPredicate<'tcx>,
) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}
