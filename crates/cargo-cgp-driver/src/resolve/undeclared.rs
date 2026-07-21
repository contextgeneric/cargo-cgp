//! Detecting a CGP capability called in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared.
//!
//! `#[cgp_fn]` and `#[cgp_impl]` lower a body into a blanket impl over a generated generic context
//! (`impl<__Context__> Describe for __Context__ where __Context__: GetName`). A capability the body
//! calls on `self` must be a `where` bound on that context — declared with `#[uses(…)]`. When the
//! body calls a capability the `#[uses]` list omits, the method cannot resolve and rustc reports a
//! vague `E0599` on `&__Context__` pointing at a transitive `HasField` bound.
//!
//! This module recognizes that shape off the compiler: the failing call sits inside a generated
//! blanket impl whose `Self` is a bare type parameter, the call names a method of a CGP capability
//! trait, and that trait is *not* among the impl's `where` bounds. It fills the rustc-free
//! [`UndeclaredCapability`] the emitter words into a `[CGP-E012]` header and `#[uses(…)]` help.

use cargo_cgp_error_processing::UndeclaredCapability;
use rustc_hir::def::DefKind;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind, HirId};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::resolve::call_site::traits_with_method;

/// Recognize an undeclared-capability failure and recover the capability trait to declare, or
/// `None` when the diagnostic is not one. Everything is keyed on `primary_span`, the failing method
/// call — never on the diagnostic's note spans (which point at *other* generated impls, such as the
/// called capability's own definition).
///
/// The three conditions, all read structurally off the compiler:
/// 1. the failing call sits inside a generated blanket impl whose `Self` is a bare type parameter —
///    the `__Context__` a `#[cgp_fn]`/`#[cgp_impl]` generates, never a concrete context. The impl is
///    found by walking up from the call's own body owner (the generated method) to its parent impl,
///    rather than by span containment: a generated impl's item span does not reliably cover its
///    body;
/// 2. the call names a method of a CGP capability trait (a consumer trait, or a
///    `#[cgp_fn]`/`#[blanket_trait]` capability trait); and
/// 3. that capability is *not* already a `where` bound of the impl — so this is a genuinely omitted
///    dependency, not a capability whose own deeper wiring fails.
pub fn detect_undeclared_capability(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    spans: &[Span],
) -> Option<UndeclaredCapability> {
    let (method, call_hir_id) = failing_call_at(tcx, primary_span)?;

    // The impl enclosing the failing call. Walk up the def-parent chain from the call's innermost
    // body owner rather than taking a single parent: a plain `#[cgp_fn]` body owner *is* the
    // generated method (parent = impl), but an `#[async_trait]` body is an async-block coroutine
    // nested inside that method, so the impl is two hops up. Stop at the first enclosing trait impl.
    let owner = tcx.hir_enclosing_body_owner(call_hir_id);
    let impl_did = enclosing_trait_impl(tcx, owner.to_def_id())?;
    let self_ty = tcx
        .impl_trait_ref(impl_did)
        .instantiate_identity()
        .skip_norm_wip()
        .self_ty();
    // A generated blanket impl's `Self` is a bare type parameter; a hand-written `impl … for
    // ConcreteContext` has an ADT `Self` and is a different (concrete-context) failure.
    if !matches!(self_ty.kind(), ty::Param(_)) {
        return None;
    }

    // The capability trait(s) declaring `method`. Several unrelated traits can share a method name
    // across modules (a `#[cgp_fn]` capability and a `#[cgp_component]` consumer both named
    // `fetch_storage_object`), so disambiguate to the one the diagnostic actually points at: the
    // "trait bound not satisfied" note spans the *failing* trait's own definition. Only if none is
    // referenced (a single unambiguous candidate) fall back to the full list.
    let candidates: Vec<DefId> = traits_with_method(tcx, method)
        .into_iter()
        .map(|(cap_did, _, _)| cap_did)
        .collect();
    let referenced: Vec<DefId> = candidates
        .iter()
        .copied()
        .filter(|&cap_did| is_referenced(tcx, cap_did, spans))
        .collect();
    let pool = if referenced.is_empty() {
        &candidates
    } else {
        &referenced
    };

    for &cap_did in pool {
        // Already a `where` bound of the impl → declared; the failure is something else.
        if impl_bounds_by(tcx, impl_did, cap_did) {
            continue;
        }
        return Some(UndeclaredCapability {
            capability: tcx.item_name(cap_did).to_string(),
        });
    }
    None
}

/// Whether the diagnostic's spans point into `trait_did`'s own definition — the "trait bound not
/// satisfied" note lands on the failing capability's `#[cgp_fn]`/`#[cgp_component]` definition,
/// which is how a same-named method on an unrelated trait in another module is told apart.
fn is_referenced(tcx: TyCtxt<'_>, trait_did: DefId, spans: &[Span]) -> bool {
    let def_span = tcx.def_span(trait_did);
    spans.iter().any(|&span| def_span.overlaps(span))
}

/// The name and `HirId` of the method call the primary span points at — matched precisely on the
/// method segment's own span, so a call nested beside it in the same expression (a sibling argument
/// of one `format!`, say) is not mistaken for the failing one.
fn failing_call_at(tcx: TyCtxt<'_>, primary_span: Span) -> Option<(Symbol, HirId)> {
    struct Finder {
        span: Span,
        found: Option<(Symbol, HirId)>,
    }
    impl<'tcx> Visitor<'tcx> for Finder {
        fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
            if let ExprKind::MethodCall(segment, ..) = expr.kind
                && segment.ident.span.overlaps(self.span)
            {
                self.found = Some((segment.ident.name, expr.hir_id));
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

/// The nearest enclosing trait impl of `did`, walking up the def-parent chain (through the
/// intervening `#[async_trait]` coroutine body and its method), or `None` if none is reached before
/// the crate root. This is what lets the detection work uniformly for a sync `#[cgp_fn]` body (impl
/// one hop up) and an async one (impl two hops up, past the async-block coroutine).
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

/// Whether `impl_did`'s `where` clause carries a trait bound of trait `trait_did` — i.e. the impl
/// already declares that capability (via `#[uses]` or a hand-written bound).
fn impl_bounds_by(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId) -> bool {
    tcx.predicates_of(impl_did)
        .predicates
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|bound| bound.skip_binder().trait_ref.def_id == trait_did)
}
