//! Unifying a call's arguments with the inputs its callee declares.

use rustc_hir::Expr;
use rustc_middle::ty::{self, GenericArgs, Ty};
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

/// Unify each argument of a call to `item_did` with the declared input it fills, in `ocx`'s
/// inference context, and return the item's signature instantiated with `item_args`.
///
/// The signature's late-bound lifetimes are instantiated as placeholders first, since relating a
/// type with an escaping bound var panics the generalizer. The first `skip_inputs` declared inputs
/// take no argument (a method call's `self` receiver is not among its arguments). Each argument
/// whose type `arg_ty` reads without typeck is unified with its input on a best-effort basis: an
/// argument that does not unify (a mis-guessed candidate, a coerced argument) only leaves the
/// parameters it would fix unknown. The caller pins what it knows in `item_args` first and reads
/// back the parameters, the bounds, or the output it needs afterward.
pub fn unify_call_inputs<'tcx>(
    ocx: &ObligationCtxt<'_, 'tcx>,
    item_did: DefId,
    item_args: GenericArgs<'tcx>,
    args: &[Expr<'tcx>],
    skip_inputs: usize,
    mut arg_ty: impl FnMut(&Expr<'tcx>) -> Option<Ty<'tcx>>,
) -> ty::FnSig<'tcx> {
    let tcx = ocx.infcx.tcx;
    let sig = tcx
        .fn_sig(item_did)
        .instantiate(tcx, item_args)
        .skip_norm_wip();
    let sig = ocx.infcx.enter_forall_and_leak_universe(sig);
    for (arg, declared) in std::iter::zip(args, sig.inputs().iter().skip(skip_inputs)) {
        if let Some(written) = arg_ty(arg) {
            let _ = ocx.eq(
                &ObligationCause::dummy(),
                ty::ParamEnv::empty(),
                *declared,
                written,
            );
        }
    }
    sig
}
