//! The by-consumer and by-capability use-site anchors: walking a trait the diagnostic names.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::anchor::{consumer_obligation, context_candidates_from_spans};
use crate::resolve::cache::ResolveCache;
use crate::resolve::call_site::contexts_at_spans;
use crate::resolve::cgp_item::{consumer_provider_trait, is_capability_trait, is_local_adt};
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
    resolve_from_named_trait(tcx, cache, spans, TraitKind::CgpConsumer)
}

/// Resolve a use-site failure by anchoring on a `#[cgp_fn]` / `#[blanket_trait]` **capability
/// trait** the diagnostic names — the counterpart of [`resolve_use_site_consumer`] for a trait that
/// is not a CGP *component* (a local blanket-impl trait with no provider trait or
/// `DelegateComponent`). Its obligation `Ctx: Capability` is walkable exactly as a consumer's is,
/// since its `Self` is the context; the result is headed `[CGP-E009] the trait …` rather than
/// `[CGP-E001] the consumer trait …` by clearing [`Resolved::consumers_are_cgp`].
///
/// This reaches the shape a capability required through a `where` **bound** or supertrait produces —
/// `fn greet_all<Context: GetName>(…)` called with a context missing the field — where the failure
/// is an `E0277` on the call with no method call on a concrete context to read. It is tried **after**
/// the [call-site anchor](crate::resolve::call_site): a *direct* method call
/// (`app.describe()`) anchors on the called method there, so its tree leads with the capability the
/// programmer actually invoked rather than with whichever sub-capability the diagnostic happens to
/// name in its spans. `None` when the diagnostic names no local capability trait, or none of the
/// candidate contexts fails one resolvably.
pub fn resolve_use_site_capability(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
) -> Option<Resolved> {
    resolve_from_named_trait(tcx, cache, spans, TraitKind::Capability)
}

/// Which local trait an anchor recovers from the diagnostic's spans.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TraitKind {
    /// A CGP consumer trait — walked as-is, headed `[CGP-E001] the consumer trait …`.
    CgpConsumer,
    /// A `#[cgp_fn]`/`#[blanket_trait]` capability trait that is not a CGP component — headed
    /// `[CGP-E009] the trait …`.
    Capability,
}

/// Recover the context and the named trait of `kind` from the diagnostic's spans, seed `Ctx: Trait`,
/// and walk it — the shared body of the two anchors above. A [`TraitKind::Capability`] result has
/// its `consumers_are_cgp` flag cleared, since such a trait is not a CGP component.
fn resolve_from_named_trait(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
    kind: TraitKind,
) -> Option<Resolved> {
    // The failing context, from a struct-definition span (an `E0599` "method not found for this
    // struct"). For the capability path this is unioned with the *expression* whose type fails (a
    // `where`-bound `E0277` on a call argument, whose context sits on no struct-definition span —
    // see [`contexts_at_spans`]); the consumer path keeps its existing struct-span recovery alone.
    let mut contexts = context_candidates_from_spans(tcx, spans);
    if kind == TraitKind::Capability {
        contexts.extend(contexts_at_spans(tcx, spans));
    }
    for trait_did in local_traits_from_spans(tcx, spans, kind) {
        // `count() == 1` is `Self` alone, so the obligation is simply `Ctx: Trait` (no params).
        if tcx.generics_of(trait_did).count() != 1 {
            continue;
        }
        for &context in &contexts {
            if !is_local_adt(context) {
                continue;
            }
            let Some(top) = consumer_obligation(tcx, context, trait_did, tcx.types.unit) else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(mut resolved) = resolve_leaves(tcx, cache, top) {
                if kind == TraitKind::Capability {
                    resolved.consumers_are_cgp = false;
                }
                return Some(resolved);
            }
        }
    }
    None
}

/// The local traits of `kind` the diagnostic's spans reference — a trait a use-site `E0599`/`E0277`
/// names in a `` `Trait` defines an item … `` note or a `required for … to implement `Trait`` note,
/// whose span points at the trait definition. A trait qualifies when it is defined in this crate and
/// its definition span contains one of the diagnostic's spans, and either it is a CGP consumer
/// ([`TraitKind::CgpConsumer`], paired with a provider via [`consumer_provider_trait`]) or a local
/// non-consumer blanket-impl trait ([`TraitKind::Capability`], the `#[cgp_fn]`/`#[blanket_trait]`
/// shape). The generated provider and getter traits and plain non-CGP traits carry neither pairing
/// nor a local blanket impl of the required shape and are filtered out.
fn local_traits_from_spans(tcx: TyCtxt<'_>, spans: &[Span], kind: TraitKind) -> Vec<DefId> {
    let mut traits = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        let did = local.to_def_id();
        if !matches!(tcx.def_kind(did), DefKind::Trait) {
            continue;
        }
        let is_consumer = consumer_provider_trait(tcx, did).is_some();
        let matches = match kind {
            TraitKind::CgpConsumer => is_consumer,
            TraitKind::Capability => !is_consumer && is_capability_trait(tcx, did),
        };
        if !matches {
            continue;
        }
        let def_span = tcx.def_span(did);
        if spans.iter().any(|&span| def_span.contains(span)) {
            traits.push(did);
        }
    }
    traits
}
