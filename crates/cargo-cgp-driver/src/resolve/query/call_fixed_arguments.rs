//! The trait arguments a call fixes, for a bound a help writes.

use rustc_hir::Expr;
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use super::{argument_ty, unify_call_inputs};

/// The type each of `params` takes in a call to `method_did` with `args`, or `None` for a parameter
/// the call leaves unknown or fixes to a type a written bound cannot hold.
///
/// The method's whole item gets fresh variables, its leading arguments are pinned to `pinned` in
/// order (`Self` first), and each argument after the first `skip_inputs` declared inputs is unified
/// with its input where [`argument_ty`](fn@argument_ty) reads its type; an argument that does not
/// unify (a coerced one) leaves its parameters unknown. A resolved type is kept unless it holds an
/// inference variable or a lifetime, or, when `params_writable` is false, a generic parameter,
/// since a `#[uses]` bound cannot name the impl's parameters while a `#[use_provider]` bound on the
/// same impl can. A kept type holds neither, so it outlives the inference context.
///
/// Precondition: each of `params` is a type parameter of the method's trait, as both callers check
/// before the call.
pub fn call_fixed_arguments<'tcx>(
    tcx: TyCtxt<'tcx>,
    method_did: DefId,
    pinned: &[Ty<'tcx>],
    args: &[Expr<'tcx>],
    skip_inputs: usize,
    params: &[ty::GenericParamDef],
    params_writable: bool,
) -> Vec<Option<Ty<'tcx>>> {
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let method_args = infcx.fresh_args_for_item(DUMMY_SP, method_did);
    let cause = ObligationCause::dummy();
    for (index, &ty) in pinned.iter().enumerate() {
        let _ = ocx.eq(&cause, param_env, method_args.type_at(index), ty);
    }
    unify_call_inputs(&ocx, method_did, method_args, args, skip_inputs, |arg| {
        argument_ty(&infcx, arg)
    });
    let _ = ocx.try_evaluate_obligations();

    params
        .iter()
        .map(|param| {
            let resolved =
                infcx.deeply_resolve_ignoring_regions(method_args.type_at(param.index as usize));
            let writable = !resolved.has_infer()
                && (params_writable || !resolved.has_param())
                && !resolved
                    .walk()
                    .any(|arg| matches!(arg.kind(), ty::GenericArgKind::Lifetime(_)));
            writable.then_some(resolved)
        })
        .collect()
}
