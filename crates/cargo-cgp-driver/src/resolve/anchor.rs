//! Recovering the starting obligation of a check failure.
//!
//! Two entry points recover the obligation differently, then feed the same
//! [walk](crate::resolve::walk): [`resolve_check_failure`] anchors on a `check_components!` entry
//! (by matching the failing diagnostic's caret to the check impl's `Self`-type span), while
//! [`resolve_use_site`] handles a consumer-method `E0599` (by recovering the context ADT from the
//! diagnostic's spans and re-checking every component that context wires).

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{Cause, Resolved};
use rustc_hir::ItemKind;
use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::cgp_item::{find_cgp_trait, is_cgp_item};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve the root cause(s) of the check failure whose diagnostic caret sits at `primary_span`,
/// or `None` if this is not a resolvable `CanUseComponent` check failure (in which case the
/// caller leaves the original diagnostic to the text-rewrite fallback). `names` supplies the
/// consumer/provider trait names the dependency tree renders CGP markers as.
pub fn resolve_check_failure(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    names: &ComponentNameMap,
) -> Option<Resolved> {
    for trait_did in tcx.all_traits_including_private() {
        let Some(super_clause) = can_use_component_supertrait(tcx, trait_did) else {
            continue;
        };

        for impl_did in tcx.all_impls(trait_did) {
            // The entry the error is about is the impl whose `Self` type carries the caret's
            // span (the macro re-spans the context type onto the entry, so they coincide).
            if impl_self_ty_span(tcx, impl_did) != Some(primary_span) {
                continue;
            }

            let trait_ref = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip();
            let concrete = super_clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));

            let Some(top) = concrete.as_trait_clause() else {
                continue;
            };
            if let Some(resolved) = resolve_leaves(tcx, top, names) {
                return Some(resolved);
            }
        }
    }
    None
}

/// Resolve the root cause(s) of a CGP wiring failure reported at a *use site* rather than a
/// `check_components!` entry — a consumer-method call (`E0599`) or any other diagnostic whose
/// obligation is not recoverable from a check impl. There is no check impl to anchor on, so the
/// context type is recovered from a diagnostic span that lands on a local struct/enum definition,
/// and every component that context wires (through its `DelegateComponent` impls) is re-checked;
/// each one that cannot be used contributes its dependency tree. `None` when no context is found
/// or no wired component fails resolvably.
pub fn resolve_use_site(
    tcx: TyCtxt<'_>,
    spans: &[Span],
    names: &ComponentNameMap,
) -> Option<Resolved> {
    let can_use_did = find_cgp_trait(tcx, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;

    // A diagnostic span can land on a provider struct as well as the real context (both are local
    // ADTs), so try each candidate and keep the first that actually wires a failing component.
    for context in context_candidates_from_spans(tcx, spans) {
        let mut causes: Vec<Cause> = Vec::new();
        let mut consumers: Vec<String> = Vec::new();
        for marker in delegated_markers(tcx, context) {
            // `Ctx: CanUseComponent<Marker, ()>` — the parameterless form, which suits the
            // components a use-site failure exercises; a component whose `()` form holds is skipped.
            let trait_ref = ty::TraitRef::new(tcx, can_use_did, [context, marker, tcx.types.unit]);
            let top: ty::PolyTraitPredicate<'_> = ty::Binder::dummy(trait_ref).upcast(tcx);
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, top, names) {
                for consumer in resolved.consumers {
                    if !consumers.contains(&consumer) {
                        consumers.push(consumer);
                    }
                }
                for cause in resolved.causes {
                    if !causes.iter().any(|c| c.key() == cause.key()) {
                        causes.push(cause);
                    }
                }
            }
        }
        if !causes.is_empty() {
            return Some(Resolved {
                context: tcx.erase_and_anonymize_regions(context).to_string(),
                consumers,
                causes,
            });
        }
    }
    None
}

/// The candidate context types of a use-site failure: every local struct or enum whose definition
/// span contains one of the diagnostic's spans — for an `E0599` method error that includes the
/// "method not found for this struct" span on the receiver's type. Each ADT is returned with
/// identity arguments (so a generic context keeps its generic form); the caller picks the one that
/// actually wires a failing component, which discards a provider struct that merely shares a span.
fn context_candidates_from_spans<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<Ty<'tcx>> {
    let mut candidates = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        let did = local.to_def_id();
        if !matches!(
            tcx.def_kind(did),
            rustc_hir::def::DefKind::Struct | rustc_hir::def::DefKind::Enum
        ) {
            continue;
        }
        let def_span = tcx.def_span(did);
        if spans.iter().any(|&span| def_span.contains(span)) {
            candidates.push(tcx.type_of(did).instantiate_identity().skip_norm_wip());
        }
    }
    candidates
}

/// The component markers a context wires, read from its `DelegateComponent<Marker>` impls — the
/// components whose use-site failure the resolver re-checks.
fn delegated_markers<'tcx>(tcx: TyCtxt<'tcx>, context: Ty<'tcx>) -> Vec<Ty<'tcx>> {
    let Some(delegate_did) = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
    else {
        return Vec::new();
    };
    let context = tcx.erase_and_anonymize_regions(context);

    let mut markers = Vec::new();
    for impl_did in tcx.all_impls(delegate_did) {
        let impl_self = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        if tcx.erase_and_anonymize_regions(impl_self) != context {
            continue;
        }
        // `DelegateComponent<Marker>` — args are `[Self, Marker]`.
        let marker = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip()
            .args
            .type_at(1);
        markers.push(tcx.erase_and_anonymize_regions(marker));
    }
    markers
}

/// The `CanUseComponent<..>` supertrait clause of `trait_did`, if it carries one — the marker
/// of a `check_components!` check trait. Anchored by DefId to `cgp_component`.
fn can_use_component_supertrait(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<ty::Clause<'_>> {
    for &(clause, _) in tcx.explicit_super_predicates_of(trait_did).skip_binder() {
        if let Some(tp) = clause.as_trait_clause()
            && is_cgp_item(
                tcx,
                tp.def_id(),
                CAN_USE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        {
            return Some(clause);
        }
    }
    None
}

/// The source span of an impl's `Self` type, e.g. the `Rectangle` in
/// `impl __CheckRectangle<..> for Rectangle {}` — which the check macro re-spans onto the
/// `check_components!` entry, so it matches the failing diagnostic's primary span.
fn impl_self_ty_span(tcx: TyCtxt<'_>, impl_did: DefId) -> Option<Span> {
    let local = impl_did.as_local()?;
    match tcx.hir_expect_item(local).kind {
        ItemKind::Impl(imp) => Some(imp.self_ty.span),
        _ => None,
    }
}
