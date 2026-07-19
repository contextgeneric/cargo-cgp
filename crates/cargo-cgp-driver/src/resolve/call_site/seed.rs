//! Building the seed obligation by unifying the call against the method's signature.

use rustc_hir::Expr;
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode, Upcast as _};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::call_site::expr_written_ty;
use crate::resolve::walk::unknowns_to_placeholders;

/// Build the seed obligation `Ctx: Consumer<…>` by unifying the call's arguments against the
/// method's own declared signature — no calling convention assumed; the signature's own use of
/// the trait's generics decides what each written argument type pins down (the rationale is in
/// the implementation document's call-site section). Every parameter of the method's item starts
/// as a fresh inference variable; `Self` is pinned to the recovered context; each argument whose
/// type the call *writes* syntactically ([`expr_written_ty`]) is unified with its declared input;
/// and each parameter left unconstrained is folded into a rigid
/// [placeholder](Ty::new_placeholder) the walk treats as unknown
/// ([`unknowns_to_placeholders`]). `None` when the consumer carries a const parameter, which this
/// recovery cannot supply.
pub(crate) fn seed_from_call<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
    consumer_did: DefId,
    method_did: DefId,
    args: &[Expr<'tcx>],
) -> Option<ty::PolyTraitPredicate<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);

    // Fresh variables for the method's whole item — for an associated fn the parent trait's
    // parameters come first, `Self` at index 0 — then pin `Self` to the recovered context.
    let method_args = infcx.fresh_args_for_item(DUMMY_SP, method_did);
    ocx.eq(
        &ObligationCause::dummy(),
        param_env,
        method_args.type_at(0),
        context,
    )
    .ok()?;

    // The declared inputs, their late-bound lifetimes instantiated as placeholders first —
    // relating a type with an escaping bound var panics the generalizer, as in the walk.
    let sig = tcx
        .fn_sig(method_did)
        .instantiate(tcx, method_args)
        .skip_norm_wip();
    let sig = infcx.enter_forall_and_leak_universe(sig);
    // `inputs()[0]` is the `self` receiver (the caller requires one), already pinned above, so the
    // call's arguments line up with the declared inputs from position 1.
    for (arg, declared) in std::iter::zip(args, sig.inputs().iter().skip(1)) {
        if let Some(written) = expr_written_ty(&infcx, arg) {
            // Best effort: a written type that does not unify (a mis-guessed consumer candidate,
            // a coerced argument) just leaves its parameter unknown — the seed is gated on
            // failing, and the walk reports nothing it cannot prove.
            let _ = ocx.eq(&ObligationCause::dummy(), param_env, *declared, written);
        }
    }
    // Propagate what the unifications imply before the variables are read back.
    let _ = ocx.try_evaluate_obligations();

    // Read the trait's parameters back out of the method args, each unresolved remainder folded
    // into a rigid placeholder so the seed can cross into the walk's fresh inference contexts.
    let mut seed_args: Vec<ty::GenericArg<'tcx>> = vec![context.into()];
    for param in &tcx.generics_of(consumer_did).own_params[1..] {
        match param.kind {
            ty::GenericParamDefKind::Type { .. } => {
                let var = method_args.type_at(param.index as usize);
                let resolved = infcx.resolve_vars_if_possible(var);
                seed_args.push(unknowns_to_placeholders(tcx, resolved).into());
            }
            // A lifetime parameter is erased everywhere in the walk; supply it erased here too.
            ty::GenericParamDefKind::Lifetime => seed_args.push(tcx.lifetimes.re_erased.into()),
            ty::GenericParamDefKind::Const { .. } => return None,
        }
    }
    let trait_ref = ty::TraitRef::new(tcx, consumer_did, seed_args);
    let seed: ty::PolyTraitPredicate<'tcx> = ty::Binder::dummy(trait_ref).upcast(tcx);
    // Erase the region variables the fresh args and written references minted, so nothing of this
    // inference context leaks into the walk's.
    Some(tcx.erase_and_anonymize_regions(seed))
}
