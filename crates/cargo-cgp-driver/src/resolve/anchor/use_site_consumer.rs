//! The by-consumer use-site anchor: walking the consumer trait the diagnostic names.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::anchor::{consumer_obligation, context_candidates_from_spans};
use crate::resolve::cache::ResolveCache;
use crate::resolve::cgp_item::{consumer_provider_trait, is_local_adt};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve a use-site failure by anchoring on the **consumer trait** the diagnostic names, rather
/// than on the components the context wires. A consumer-method call names its consumer trait in a
/// note (`` `CanGreet` defines an item `greet` ``), whose span points at the trait definition; when
/// that trait is a local, non-generic CGP consumer, this recovers it and walks the real obligation
/// `Ctx: Consumer` directly — no marker, no `CanUseComponent`/`IsProviderFor` detour.
///
/// This is what reaches a **namespace-joined** context, whose concrete wiring lives in the joined
/// namespace and not in its own `DelegateComponent` impls.
/// [`resolve_use_site`](super::resolve_use_site)'s per-component re-check finds only the
/// namespace's blanket forwarding key (a bare parameter, skipped) and yields nothing; the walk
/// started here instead descends `Ctx: Consumer → Provider: ProviderTrait<Ctx, …>` and lets the
/// delegate normalize *through* the namespace on its own, so no per-context enumeration of the
/// namespace's wiring is needed. It is deliberately tried after `resolve_use_site`, so a
/// directly-wired context keeps its existing recovery.
///
/// Restricted to a consumer whose only generic is `Self`, so `Ctx: Consumer` forms without the
/// component parameters a use site does not carry — a generic consumer (`CanHandle<Code, Input>`) is
/// left to decline. `None` when the diagnostic names no local CGP consumer trait, or none of the
/// candidate contexts fails one resolvably.
pub fn resolve_use_site_consumer(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
) -> Option<Resolved> {
    for consumer_did in local_cgp_consumer_traits_from_spans(tcx, spans) {
        // `count() == 1` is `Self` alone, so the obligation is simply `Ctx: Consumer` (no params).
        if tcx.generics_of(consumer_did).count() != 1 {
            continue;
        }
        for context in context_candidates_from_spans(tcx, spans) {
            if !is_local_adt(context) {
                continue;
            }
            let Some(top) = consumer_obligation(tcx, context, consumer_did, tcx.types.unit) else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, cache, top) {
                return Some(resolved);
            }
        }
    }
    None
}

/// The local CGP consumer traits the diagnostic's spans reference — the trait a consumer-method
/// `E0599` names in its "`Trait` defines an item …" note. A trait is a candidate when it is defined
/// in this crate, its definition span contains one of the diagnostic's spans, and it is a CGP
/// consumer (it pairs with a provider trait through its blanket impl, via [`consumer_provider_trait`]);
/// the generated provider trait, getter traits, and non-CGP traits carry no such pairing and are
/// filtered out.
fn local_cgp_consumer_traits_from_spans(tcx: TyCtxt<'_>, spans: &[Span]) -> Vec<DefId> {
    let mut traits = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        let did = local.to_def_id();
        if !matches!(tcx.def_kind(did), DefKind::Trait) {
            continue;
        }
        if consumer_provider_trait(tcx, did).is_none() {
            continue;
        }
        let def_span = tcx.def_span(did);
        if spans.iter().any(|&span| def_span.contains(span)) {
            traits.push(did);
        }
    }
    traits
}
