//! Recovering the starting obligation of a check failure.
//!
//! Three entry points recover the obligation differently, then feed the same
//! [walk](crate::resolve::walk): [`resolve_check_failure`] anchors on a `check_components!` entry
//! (by matching the failing diagnostic's caret to the check impl's `Self`-type span);
//! [`resolve_impl_site`] handles a wiring failure surfaced *inside a hand-written `impl Trait for
//! Context` block* (by recovering the exact failing obligation — with its concrete component
//! parameters — from the impl's CGP consumer supertrait); and [`resolve_use_site`] handles a
//! consumer-method `E0599` (by recovering the context ADT from the diagnostic's spans and
//! re-checking the parameterless form of every component that context wires).

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{Cause, Resolved};
use rustc_hir::ItemKind;
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::cgp_item::{find_cgp_trait, is_cgp_item, is_provider_trait};
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

/// Resolve the root cause(s) of a CGP wiring failure reported *inside a hand-written `impl Trait
/// for Context` block* — the shape a wrapper trait that carries a CGP consumer trait as a
/// supertrait produces when it is implemented directly on a concrete context (the transfer
/// example's per-endpoint `impl CanHandleApiSend<Api> for MockApp`, added to bound a future
/// `Send`). Such a failure anchors on neither a `check_components!` entry nor a consumer-method
/// call, and its caret sits on the impl rather than on the context's own type definition, so
/// [`resolve_use_site`] cannot recover the context from a struct-definition span.
///
/// This entry recovers the context from the enclosing impl's `Self` type, and — crucially — the
/// *exact* failing obligation from the impl's CGP consumer supertrait, so a generic component
/// carries its concrete parameter (`CanCalculateArea<Rectangle>`, not the `()` form the use-site
/// re-check would substitute). It reconstructs the `Ctx: CanUseComponent<Marker, Params>`
/// obligation that supertrait stands for and walks it exactly as a check entry would, yielding an
/// identical root-cause tree. `None` when no enclosing impl on a local context carries an unmet,
/// reconstructable CGP consumer supertrait.
pub fn resolve_impl_site(
    tcx: TyCtxt<'_>,
    spans: &[Span],
    names: &ComponentNameMap,
) -> Option<Resolved> {
    for impl_did in enclosing_trait_impls(tcx, spans) {
        // Safe because `enclosing_trait_impls` keeps only `of_trait` impls.
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
        // Only a local struct/enum is a context whose wiring we can re-check; skip an impl on a
        // foreign type (e.g. `impl … for Router<Arc<App>>`) or a type parameter.
        if !is_local_adt(context) {
            continue;
        }

        let mut causes: Vec<Cause> = Vec::new();
        let mut consumers: Vec<String> = Vec::new();
        // Each supertrait the impl's trait carries, instantiated for this impl's `Self`. A CGP
        // consumer trait among them that does not hold is the wiring failure the impl surfaces.
        for &(clause, _) in tcx
            .explicit_super_predicates_of(trait_ref.def_id)
            .skip_binder()
        {
            let concrete = clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));
            let Some(sup) = concrete.as_trait_clause() else {
                continue;
            };
            if tcx.erase_and_anonymize_regions(sup.skip_binder().self_ty()) != context {
                continue;
            }
            if holds(tcx, sup) {
                continue;
            }
            let Some(top) = consumer_can_use_obligation(tcx, context, sup) else {
                continue;
            };
            let Some(resolved) = resolve_leaves(tcx, top, names) else {
                continue;
            };
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

        if !causes.is_empty() {
            return Some(Resolved {
                context: context.to_string(),
                consumers,
                causes,
            });
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

/// The local trait-impl blocks (`impl Trait for Ty { … }`) whose source span contains one of the
/// diagnostic's spans — the impls a wiring failure can surface inside, whether the caret lands on
/// the impl header or deep in a method body. A trait impl (not an inherent one) is required
/// because the recovery reads the impl's trait supertraits. The *full* item span is taken from HIR
/// rather than `def_span`, which for an impl covers only the header and would miss a caret inside
/// the body (a forwarding `self.method(..)` call, say).
fn enclosing_trait_impls(tcx: TyCtxt<'_>, spans: &[Span]) -> Vec<DefId> {
    let mut impls = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_span = tcx.hir_expect_item(local).span;
        if spans.iter().any(|&span| impl_span.contains(span)) {
            impls.push(local.to_def_id());
        }
    }
    impls
}

/// Whether `ty` is a struct or enum defined in the crate being compiled — the only kind of type
/// whose wiring the resolver re-checks as a context.
fn is_local_adt(ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if def.did().is_local())
}

/// Reconstruct the `Ctx: CanUseComponent<Marker, Params>` obligation a CGP consumer-trait bound
/// (`Ctx: CanCalculateArea<Rectangle>`) stands for, so it can be walked exactly as a
/// `check_components!` entry is — with the component's concrete parameters preserved. Returns
/// `None` when `consumer` is not a recognizable CGP consumer trait (its blanket impl does not bound
/// its own context on a provider trait, or that provider trait carries no `IsProviderFor` marker),
/// so a plain supertrait such as `Send` is skipped rather than mis-walked.
fn consumer_can_use_obligation<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
    consumer: ty::PolyTraitPredicate<'tcx>,
) -> Option<ty::PolyTraitPredicate<'tcx>> {
    let can_use_did = find_cgp_trait(tcx, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;
    let consumer_ref = consumer.skip_binder().trait_ref;
    let provider_did = consumer_provider_trait(tcx, consumer_ref.def_id)?;
    let marker = provider_marker(tcx, provider_did)?;

    // The component's extra parameters are grouped into the `Params` slot exactly as CGP groups
    // them: none as the unit `()`, a single one bare, several as a tuple.
    let extra: Vec<Ty<'tcx>> = consumer_ref.args.types().skip(1).collect();
    let params = match extra.as_slice() {
        [] => tcx.types.unit,
        [single] => *single,
        many => Ty::new_tup(tcx, many),
    };

    let trait_ref = ty::TraitRef::new(tcx, can_use_did, [context, marker, params]);
    Some(ty::Binder::dummy(trait_ref).upcast(tcx))
}

/// The provider trait a consumer trait pairs with, found through the consumer's blanket impl
/// `impl<C> Consumer for C where C: Provider<C>`: the `where`-bound whose self type is the impl's
/// own self is the provider trait. This is the per-consumer form of the inversion
/// [`component_map`](crate::component_map) performs across the whole trait graph.
fn consumer_provider_trait(tcx: TyCtxt<'_>, consumer_did: DefId) -> Option<DefId> {
    for &impl_did in tcx.trait_impls_of(consumer_did).blanket_impls() {
        let impl_self = tcx.type_of(impl_did).skip_binder();
        for &(clause, _) in tcx.predicates_of(impl_did).predicates {
            let Some(tp) = clause.as_trait_clause() else {
                continue;
            };
            let trait_ref = tp.skip_binder().trait_ref;
            if trait_ref.self_ty() == impl_self && is_provider_trait(tcx, trait_ref.def_id) {
                return Some(trait_ref.def_id);
            }
        }
    }
    None
}

/// The component marker a provider trait keys, read from the `IsProviderFor<Marker, …>` supertrait
/// every provider trait carries — the same second-argument marker
/// [`component_map`](crate::component_map) reads. Anchored by DefId to `cgp_component`.
fn provider_marker<'tcx>(tcx: TyCtxt<'tcx>, provider_did: DefId) -> Option<Ty<'tcx>> {
    for &(clause, _) in tcx.explicit_super_predicates_of(provider_did).skip_binder() {
        let Some(tp) = clause.as_trait_clause() else {
            continue;
        };
        let trait_ref = tp.skip_binder().trait_ref;
        if is_cgp_item(
            tcx,
            trait_ref.def_id,
            IS_PROVIDER_FOR_TRAIT,
            CGP_COMPONENT_CRATE,
        ) {
            return trait_ref.args.get(1).and_then(|arg| arg.as_type());
        }
    }
    None
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
