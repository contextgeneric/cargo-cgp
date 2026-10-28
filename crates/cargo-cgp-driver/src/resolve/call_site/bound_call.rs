//! Anchoring a failing bound of a generic function at the call that instantiates it.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind, QPath};
use rustc_infer::infer::{InferCtxt, TyCtxtInferExt as _};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::call_site::{data_expr_ty, declared_binding_ty, is_about_call};
use crate::resolve::cgp_item::{
    carries_wiring, is_blanket_trait, is_consumer_trait, is_context_adt,
};
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::query::{
    WrittenArg, expr_written_ty, holds, lower_hir_ty, unknowns_to_placeholders, written_args,
};
use crate::resolve::walk::resolve_leaves;

/// Resolve a failing `where` bound of a generic function at a call that names it by path
/// (`fail::<App>()` for `fn fail<Context: CanRaiseError<String>>()`). The function's own bounds,
/// instantiated with the type arguments the call writes in its turbofish and those its arguments
/// write, are the obligations the call asserts, so a CGP consumer trait or a
/// `#[cgp_fn]`/`#[blanket_trait]` blanket trait among them, on a context, that does not hold is
/// seeded and walked as a call through the trait would be. A consumer trait heads the block as
/// `[CGP-E001]`, a blanket trait as `[CGP-E003]`. `None` when no such call sits at the
/// diagnostic's primary spans, or no bound on a context fails resolvably.
pub fn resolve_bound_call(cx: ResolveCtx<'_, '_>, spans: &[Span]) -> Option<Resolved> {
    let tcx = cx.tcx;
    for call in generic_calls_at(tcx, spans) {
        for top in failing_context_bounds(tcx, &call) {
            let trait_did = top.skip_binder().def_id();
            let blanket = !is_consumer_trait(tcx, trait_did);
            if let Some(mut resolved) = resolve_leaves(cx, top) {
                if blanket {
                    resolved.consumers_are_cgp = false;
                    resolved.subject_is_context = carries_wiring(tcx, top.skip_binder().self_ty());
                }
                return Some(resolved);
            }
        }
    }
    None
}

/// One call to a generic function named by path, found at the diagnostic's primary spans.
struct GenericCall<'tcx> {
    fn_did: DefId,
    callee: &'tcx rustc_hir::Path<'tcx>,
    args: &'tcx [Expr<'tcx>],
}

/// Every call in a local body to a free generic function named by path (`f::<T>(…)`, `f(…)`) that
/// one of the diagnostic's primary spans is about ([`is_about_call`]); a caret on the turbofish
/// is inside the callee, so it is about the call.
fn generic_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<GenericCall<'tcx>> {
    struct Finder<'a, 'tcx> {
        tcx: TyCtxt<'tcx>,
        spans: &'a [Span],
        calls: Vec<GenericCall<'tcx>>,
    }
    impl<'tcx> Visitor<'tcx> for Finder<'_, 'tcx> {
        fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
            if let ExprKind::Call(callee, args) = expr.kind
                && let ExprKind::Path(QPath::Resolved(None, path)) = callee.kind
                && let Res::Def(DefKind::Fn, fn_did) = path.res
                && !self.tcx.generics_of(fn_did).own_params.is_empty()
                && self
                    .spans
                    .iter()
                    .any(|&span| is_about_call(span, expr.span, args))
            {
                self.calls.push(GenericCall {
                    fn_did,
                    callee: path,
                    args,
                });
            }
            intravisit::walk_expr(self, expr);
        }
    }
    let mut finder = Finder {
        tcx,
        spans,
        calls: Vec::new(),
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.calls
}

/// The bounds of `call`'s function on a CGP consumer or blanket trait whose `Self` is a context,
/// instantiated for the call, that do not hold. The function's parameters start as fresh variables;
/// its turbofish's written types and its arguments' types are unified in, and whatever stays
/// unknown becomes a rigid placeholder the walk never reports on.
fn failing_context_bounds<'tcx>(
    tcx: TyCtxt<'tcx>,
    call: &GenericCall<'tcx>,
) -> Vec<ty::PolyTraitClause<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let fn_args = infcx.fresh_args_for_item(DUMMY_SP, call.fn_did);

    // The turbofish fills the explicit type parameters in order; `impl Trait` arguments' synthetic
    // parameters take none, and a const argument is left unknown.
    if let Some(written) = written_args(tcx, call.callee) {
        let explicit = tcx
            .generics_of(call.fn_did)
            .own_params
            .iter()
            .filter(|param| !matches!(param.kind, ty::GenericParamDefKind::Lifetime))
            .filter(|param| !param.kind.is_synthetic());
        for (param, written) in explicit.zip(written) {
            if let WrittenArg::Type(ty) = written {
                let _ = ocx.eq(
                    &ObligationCause::dummy(),
                    param_env,
                    fn_args.type_at(param.index as usize),
                    ty,
                );
            }
        }
    }
    let sig = tcx
        .fn_sig(call.fn_did)
        .instantiate(tcx, fn_args)
        .skip_norm_wip();
    let sig = infcx.enter_forall_and_leak_universe(sig);
    for (arg, declared) in std::iter::zip(call.args, sig.inputs()) {
        if let Some(written) = arg_ty(&infcx, arg) {
            let _ = ocx.eq(&ObligationCause::dummy(), param_env, *declared, written);
        }
    }
    let _ = ocx.try_evaluate_obligations();

    tcx.clauses_of(call.fn_did)
        .instantiate(tcx, fn_args)
        .into_iter()
        .filter_map(|(clause, _)| {
            let clause = infcx.deeply_resolve_ignoring_regions(clause.skip_norm_wip());
            let clause = tcx.erase_and_anonymize_regions(unknowns_to_placeholders(tcx, clause));
            clause.as_trait_clause()
        })
        .filter(|bound| {
            let self_ty = bound.skip_binder().self_ty();
            let trait_did = bound.skip_binder().def_id();
            is_context_adt(tcx, self_ty)
                && !self_ty.has_param()
                && !self_ty.has_placeholders()
                && (is_consumer_trait(tcx, trait_did) || is_blanket_trait(tcx, trait_did))
        })
        .filter(|bound| !holds(tcx, *bound))
        .collect()
}

/// The type an argument expression has, as far as it can be read without typeck: written by its own
/// syntax ([`expr_written_ty`]), a binding's declared type with its references
/// ([`declared_binding_ty`]), a reference to such a value, or a value a binding or a data operation
/// produced ([`data_expr_ty`]).
fn arg_ty<'tcx>(infcx: &InferCtxt<'tcx>, arg: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    if let Some(ty) = expr_written_ty(infcx, arg) {
        return Some(ty);
    }
    if let ExprKind::Path(QPath::Resolved(None, path)) = arg.kind
        && let Res::Local(binding) = path.res
        && let Some(declared) = declared_binding_ty(infcx.tcx, binding)
    {
        return lower_hir_ty(infcx.tcx, declared);
    }
    if let ExprKind::AddrOf(_, mutbl, inner) = arg.kind {
        return Some(Ty::new_ref(
            infcx.tcx,
            infcx.tcx.lifetimes.re_erased,
            arg_ty(infcx, inner)?,
            mutbl,
        ));
    }
    data_expr_ty(infcx.tcx, arg)
}
