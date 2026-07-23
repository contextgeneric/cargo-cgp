//! Detecting a higher-order provider that calls an inner provider it never imported.
//!
//! A higher-order provider names an inner provider as a generic parameter and invokes it as an
//! associated function on the context — `InnerCalculator::area(self)`. For that call to resolve, the
//! inner parameter must carry the provider-trait bound `InnerCalculator: AreaCalculator<Self>`,
//! declared idiomatically with `#[use_provider(InnerCalculator: AreaCalculator)]`. When the import is
//! forgotten, the parameter is unbounded and rustc reports a vague `E0599` — "no associated function
//! `area` found for type parameter `InnerCalculator` … due to unsatisfied trait bounds" — whose
//! suggestion leaks the generated `__Context__` and offers the *consumer* trait as a bound, the wrong
//! fix for a higher-order provider.
//!
//! This module recognizes that shape off the compiler: the failing call is an associated-function
//! call `Param::method(…)` whose `Param` is a generic parameter of an enclosing provider-trait impl,
//! the called method belongs to a CGP *provider* trait, and `Param` is not bounded by that provider
//! trait. It fills the rustc-free [`MissingUseProvider`] the emitter words into a `[CGP-E016]` header
//! and `#[use_provider(…)]` help.

use cargo_cgp_error_processing::MissingUseProvider;
use rustc_hir::def::DefKind;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind, QPath, TyKind};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::is_provider_trait;

/// Recognize a missing-`#[use_provider]` failure and recover the inner provider and the provider
/// trait to import it as, or `None` when the diagnostic is not one. Everything is keyed on
/// `primary_span`, the failing associated-function call.
pub fn detect_missing_use_provider(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<MissingUseProvider> {
    let (inner, inner_did, method, call_hir_id) = failing_assoc_fn_call_at(tcx, primary_span)?;

    // `inner` must be a generic type parameter — a higher-order provider's inner-provider slot — not
    // a concrete type (whose associated-function failure is a different mistake).
    if !matches!(tcx.def_kind(inner_did), DefKind::TyParam) {
        return None;
    }

    // The enclosing trait impl, and its own provider trait (a higher-order provider implements one).
    let owner = tcx.hir_enclosing_body_owner(call_hir_id);
    let impl_did = enclosing_trait_impl(tcx, owner.to_def_id())?;
    let impl_trait_did = tcx.impl_trait_ref(impl_did).skip_binder().def_id;

    // The provider trait the call's method belongs to: the enclosing impl's own trait when it
    // declares the method (a higher-order provider wrapping the same component — the common case),
    // else the sole provider trait that declares it.
    let provider_did = if is_provider_trait(tcx, impl_trait_did)
        && trait_has_method(tcx, impl_trait_did, method)
    {
        impl_trait_did
    } else {
        let mut candidates = tcx.all_traits_including_private().filter(|&trait_did| {
            is_provider_trait(tcx, trait_did) && trait_has_method(tcx, trait_did, method)
        });
        let only = candidates.next()?;
        if candidates.next().is_some() {
            return None;
        }
        only
    };

    // If `inner` is already bounded by that provider trait, the call would resolve — so this `E0599`
    // is some other failure, not a forgotten import.
    if impl_bounds_param_by(tcx, impl_did, provider_did, &inner) {
        return None;
    }

    Some(MissingUseProvider {
        inner,
        provider_trait: tcx.item_name(provider_did).to_string(),
    })
}

/// The inner-provider name, its `DefId`, the called method, and the call expression's `HirId` for
/// the associated-function call `Param::method(…)` the primary span points at — matched precisely on
/// the method segment's own span.
fn failing_assoc_fn_call_at(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<(String, DefId, rustc_span::Symbol, rustc_hir::HirId)> {
    struct Finder {
        span: Span,
        found: Option<(String, DefId, rustc_span::Symbol, rustc_hir::HirId)>,
    }
    impl<'tcx> Visitor<'tcx> for Finder {
        fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
            if let ExprKind::Call(callee, _) = expr.kind
                && let ExprKind::Path(QPath::TypeRelative(qself, segment)) = callee.kind
                && segment.ident.span.overlaps(self.span)
                && let TyKind::Path(QPath::Resolved(_, path)) = qself.kind
                && let Some(param_did) = path.res.opt_def_id()
            {
                let name = path
                    .segments
                    .last()
                    .map(|seg| seg.ident.name.to_string())
                    .unwrap_or_default();
                self.found = Some((name, param_did, segment.ident.name, expr.hir_id));
            }
            intravisit::walk_expr(self, expr);
        }
    }
    let mut finder = Finder {
        span: primary_span,
        found: None,
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.found
}

/// Whether `trait_did` declares an associated function named `method`.
fn trait_has_method(tcx: TyCtxt<'_>, trait_did: DefId, method: rustc_span::Symbol) -> bool {
    tcx.associated_items(trait_did)
        .filter_by_name_unhygienic(method)
        .any(|item| matches!(item.kind, ty::AssocKind::Fn { .. }))
}

/// The nearest enclosing trait impl of `did`, walking up the def-parent chain (as
/// `resolve::undeclared` does), or `None` before the crate root.
fn enclosing_trait_impl(tcx: TyCtxt<'_>, mut did: DefId) -> Option<DefId> {
    loop {
        if matches!(tcx.def_kind(did), DefKind::Impl { of_trait: true }) {
            return Some(did);
        }
        let parent = tcx.opt_parent(did)?;
        if parent == did {
            return None;
        }
        did = parent;
    }
}

/// Whether `impl_did`'s `where` clause bounds a type parameter named `param` by trait `trait_did` —
/// i.e. the inner provider is already imported (via `#[use_provider]` or a hand-written bound).
fn impl_bounds_param_by(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId, param: &str) -> bool {
    tcx.predicates_of(impl_did)
        .predicates
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|bound| {
            let bound = bound.skip_binder();
            bound.trait_ref.def_id == trait_did
                && matches!(bound.self_ty().kind(), ty::Param(p) if p.name.as_str() == param)
        })
}
