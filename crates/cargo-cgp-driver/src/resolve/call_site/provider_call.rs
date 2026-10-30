//! Anchoring a failing call to a provider trait's method, made through the provider's own path.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::{Expr, ExprKind, HirId, QPath};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::resolve::call_site::{CallForm, is_about_call, seed_from_call};
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::query::{
    carries_wiring, holds, is_provider_trait, local_body_exprs, lower_hir_ty, peel_hir_refs,
    trait_items_named,
};
use crate::resolve::walk::resolve_provider_leaves;

/// Resolve a failing call to a provider trait's method through the provider's own path
/// (`PipeHandlers::<…>::compute(&(), …)`, `Add::try_compute(&App, …)`), which no consumer trait
/// stands between: the provider is named by the path, the provider trait's context and component
/// parameters by unifying the call's arguments against the method's declared signature, as for any
/// call, and the seeded `Provider: ProviderTrait<Ctx, …>` is walked as a `#[check_providers]`
/// entry's is. The result is the provider class, `[CGP-E002]`, since the call asserts a provider
/// trait. Only a provider trait in scope at the call is a candidate, as the compiler's own lookup
/// considers only those. `None` when no such call sits at the diagnostic's primary spans, its
/// provider or context cannot be typed, or no root cause is found.
pub fn resolve_provider_call(cx: ResolveCtx<'_, '_>, spans: &[Span]) -> Option<Resolved> {
    let tcx = cx.tcx;
    for call in provider_calls_at(tcx, spans) {
        let Some(provider) = call.provider else {
            continue;
        };
        let in_scope = tcx.in_scope_traits(call.scope).unwrap_or_default();
        let candidates =
            provider_fns_named(tcx, call.name)
                .into_iter()
                .filter(|&(trait_did, _)| {
                    in_scope
                        .iter()
                        .any(|candidate| candidate.def_id == trait_did)
                });
        for (trait_did, fn_did) in candidates {
            let Some(top) =
                seed_from_call(tcx, provider, trait_did, fn_did, call.args, CallForm::Path)
            else {
                continue;
            };
            // The provider trait's context is its first type argument after `Self`; the walk is
            // rooted there, so a context the call leaves unknown cannot be walked.
            let Some(context) = top
                .skip_binder()
                .trait_ref
                .args
                .iter()
                .skip(1)
                .find_map(|arg| arg.as_type())
            else {
                continue;
            };
            if context.has_placeholders() || holds(tcx, top) {
                continue;
            }
            if let Some(mut resolved) = resolve_provider_leaves(cx, top, context) {
                // The context argument is a context only when it carries wiring; a unit `()`
                // passed to a dispatcher is named plainly.
                resolved.subject_is_context = carries_wiring(tcx, context);
                return Some(resolved);
            }
        }
    }
    None
}

/// One call by a type-relative path (`T::f(…)`) found at the diagnostic's primary spans: the
/// function's name, the type the path names (when it can be read), the call's arguments, and the
/// callee, for which the compiler records the traits in scope.
struct ProviderCall<'tcx> {
    name: Symbol,
    provider: Option<Ty<'tcx>>,
    args: &'tcx [Expr<'tcx>],
    scope: HirId,
}

/// Every call by a type-relative path in a local body that one of the diagnostic's primary spans is
/// about, by the rule the call-site anchor applies ([`is_about_call`]), outermost first.
fn provider_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<ProviderCall<'tcx>> {
    local_body_exprs(tcx)
        .filter_map(|expr| {
            if let ExprKind::Call(callee, args) = expr.kind
                && let ExprKind::Path(QPath::TypeRelative(qself, segment)) = callee.kind
                && spans
                    .iter()
                    .any(|&span| is_about_call(span, expr.span, args))
            {
                Some(ProviderCall {
                    name: segment.ident.name,
                    provider: lower_hir_ty(tcx, peel_hir_refs(qself)),
                    args,
                    scope: callee.hir_id,
                })
            } else {
                None
            }
        })
        .collect()
}

/// The CGP provider traits that declare an associated fn named `name`, each with that fn's `DefId`.
/// A provider method takes its context as an argument rather than as `self`, so every fn counts.
fn provider_fns_named(tcx: TyCtxt<'_>, name: Symbol) -> Vec<(DefId, DefId)> {
    trait_items_named(tcx, name, |item| {
        matches!(item.kind, ty::AssocKind::Fn { .. })
    })
    .filter(|&(trait_did, _)| is_provider_trait(tcx, trait_did))
    .collect()
}
