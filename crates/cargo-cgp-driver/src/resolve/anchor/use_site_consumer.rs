//! The by-consumer and by-blanket-trait use-site anchors: walking a trait the diagnostic names.

use cargo_cgp_error_processing::{Resolved, ResolvedClass};
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::anchor::{consumer_obligation, context_candidates_from_spans};
use crate::resolve::call_site::contexts_at_spans;
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::query::{
    carries_wiring, consumer_provider_trait, holds, is_blanket_trait, is_local_adt,
};
use crate::resolve::walk::resolve_leaves;

/// Resolve a use-site failure by anchoring on the **consumer trait** the diagnostic names, which
/// reaches a namespace-joined context. Tried after [`resolve_use_site`](super::resolve_use_site),
/// and restricted to a local consumer whose only generic is `Self`. `None` when the diagnostic
/// names no such trait, or no candidate context fails one resolvably.
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/anchors.md`.
pub fn resolve_use_site_consumer(cx: ResolveCtx<'_, '_>, spans: &[Span]) -> Option<Resolved> {
    resolve_from_named_trait(cx, spans, TraitKind::CgpConsumer)
}

/// Resolve a use-site failure by anchoring on a `#[cgp_fn]` / `#[blanket_trait]` **blanket trait**
/// the diagnostic names, headed `[CGP-E003]` by changing [`Resolved::class`]. Tried after the
/// call-site anchor. `None` when the diagnostic names no local blanket trait, or no candidate
/// context fails one resolvably.
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/anchors.md`.
pub fn resolve_use_site_blanket_trait(cx: ResolveCtx<'_, '_>, spans: &[Span]) -> Option<Resolved> {
    resolve_from_named_trait(cx, spans, TraitKind::Blanket)
}

/// Which local trait an anchor recovers from the diagnostic's spans.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TraitKind {
    /// A CGP consumer trait: walked as-is, headed `[CGP-E001] the consumer trait …`.
    CgpConsumer,
    /// A `#[cgp_fn]`/`#[blanket_trait]` blanket trait that is not a CGP component: headed
    /// `[CGP-E003] the trait …`.
    Blanket,
}

/// Recover the context and the named trait of `kind` from the diagnostic's spans, seed
/// `Ctx: Trait`, and walk it: the shared body of the two anchors above. A [`TraitKind::Blanket`]
/// result has the wrapper class, since such a trait is not a CGP component.
fn resolve_from_named_trait(
    cx: ResolveCtx<'_, '_>,
    spans: &[Span],
    kind: TraitKind,
) -> Option<Resolved> {
    let tcx = cx.tcx;
    // The failing context, from a struct-definition span (an `E0599` "method not found for this
    // struct"), unioned with the *expression* whose type fails (a bound's `E0277` on a call
    // argument, whose context sits on no struct-definition span; see [`contexts_at_spans`]).
    let mut contexts = context_candidates_from_spans(tcx, spans);
    contexts.extend(contexts_at_spans(tcx, spans));
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
            if let Some(mut resolved) = resolve_leaves(cx, top) {
                if kind == TraitKind::Blanket {
                    resolved.class = ResolvedClass::Wrapper;
                    resolved.subject_is_context = carries_wiring(tcx, context);
                }
                return Some(resolved);
            }
        }
    }
    None
}

/// The local traits of `kind` the diagnostic's spans reference: a trait a use-site `E0599`/`E0277`
/// names in a `` `Trait` defines an item … `` note or a `required for … to implement `Trait`` note,
/// whose span points at the trait definition. A trait qualifies when it is defined in this crate
/// and its definition span contains one of the diagnostic's spans, and either it is a CGP consumer
/// ([`TraitKind::CgpConsumer`], paired with a provider via [`consumer_provider_trait`]) or a local
/// non-consumer blanket-impl trait ([`TraitKind::Blanket`], the `#[cgp_fn]`/`#[blanket_trait]`
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
            TraitKind::Blanket => !is_consumer && is_blanket_trait(tcx, did),
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
