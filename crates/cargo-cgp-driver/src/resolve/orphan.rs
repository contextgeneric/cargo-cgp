//! Recognizing an orphan-rule namespace registration and recovering what it registered.
//!
//! Registering wiring into a namespace lowers to `impl<Param> Namespace<Param> for Key`, and Rust's
//! orphan rule rejects it with `E0210` (or its sibling `E0117`) when both the namespace trait and
//! the key are foreign — a downstream crate registering into an upstream namespace it does not own,
//! keyed on an upstream component it does not own either. The raw diagnostic names the machinery
//! parameter (`__Components__` from a `#[default_impl]`/`#[prefix]`, `__Table__` from a
//! `cgp_namespace!` re-open) rather than the namespace and key the programmer wrote.
//!
//! [`classify_orphan_conflict`] recovers those from the compiler, not the error text: it finds the
//! offending impl at the caret, reads its trait and `Self`, and — anchoring the namespace trait by
//! its structural [namespace-lookup fingerprint](crate::resolve::cgp_item::is_namespace_lookup_trait)
//! rather than by name — words the collision into the rustc-free
//! [`OrphanConflict`]. Because we only run on an `E0210`/`E0117`, which rustc emits *only* for a
//! genuine orphan, matching the caret to a foreign-namespace-trait-for-foreign-key impl is the whole
//! confirmation: rustc has already proved the orphan, and the caret already sits on the impl it
//! proved it for.

use cargo_cgp_error_processing::{OrphanConflict, OrphanTrigger, WiringKey};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::is_namespace_lookup_trait;
use crate::resolve::conflict::describe_key;

/// One local impl of a foreign namespace-lookup trait for a foreign key — a candidate orphan
/// registration.
struct OrphanImpl<'tcx> {
    impl_did: DefId,
    def_span: Span,
    trait_did: DefId,
    key: Ty<'tcx>,
}

/// Classify an `E0210`/`E0117` at `primary_span` as an orphan-rule namespace registration, or
/// `None` when no such impl sits at the caret (so the caller leaves the diagnostic to the ordinary
/// fallback). The recovered impl is a *foreign* namespace-lookup trait implemented locally for a
/// *foreign* key; the trigger is read from the impl's own machinery parameter name so the fix can
/// be worded for the construct the programmer wrote.
pub fn classify_orphan_conflict(tcx: TyCtxt<'_>, primary_span: Span) -> Option<OrphanConflict> {
    let candidates = orphan_impls(tcx);

    // The offending impl is the one the caret sits on, matched by source range (the macro re-spans
    // its generated impl onto the invocation, whose range the coherence caret shares). When the
    // caret lands on none of them — a span shape the match does not reach — a lone candidate is
    // still unambiguous, so fall back to it; more than one leaves the choice ambiguous and declines.
    let orphan = candidates
        .iter()
        .find(|candidate| range_overlaps(candidate.def_span, primary_span))
        .or(match candidates.as_slice() {
            [only] => Some(only),
            _ => None,
        })?;

    let namespace = tcx.item_name(orphan.trait_did).to_string();
    let key = describe_key(tcx, orphan.key, orphan.impl_did)?;
    // A blanket key is a namespace *join*, not a registration; the orphan class is only a component
    // or path key, so decline anything else rather than word a nonsensical fix.
    if matches!(key, WiringKey::Blanket(_)) {
        return None;
    }

    Some(OrphanConflict {
        namespace,
        key,
        trigger: orphan_trigger(tcx, orphan.impl_did),
    })
}

/// Every local impl of a foreign namespace-lookup trait for a foreign key. These are exactly the
/// impls the orphan rule rejects for a namespace registration, whichever construct generated them.
fn orphan_impls(tcx: TyCtxt<'_>) -> Vec<OrphanImpl<'_>> {
    let mut impls = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_did = local.to_def_id();
        let trait_ref = tcx.impl_trait_ref(impl_did).skip_binder();
        let trait_did = trait_ref.def_id;
        // Foreign namespace trait — a local one would be no orphan (you may implement your own
        // trait for anything). Recognized by the single-`Delegate` fingerprint, so a downstream
        // crate's own namespace trait is treated the same as CGP's built-in `DefaultNamespace`.
        if trait_did.is_local() || !is_namespace_lookup_trait(tcx, trait_did) {
            continue;
        }
        // Foreign key — a bare component marker or a `PathCons<…>` path, both defined in another
        // crate. A local key would be covered and again no orphan.
        let key = trait_ref.self_ty();
        if !matches!(key.kind(), ty::Adt(def, _) if !def.did().is_local()) {
            continue;
        }
        impls.push(OrphanImpl {
            impl_did,
            def_span: tcx.def_span(impl_did),
            trait_did,
            key,
        });
    }
    impls
}

/// Which construct generated the impl, read from its own machinery type parameter: a
/// `cgp_namespace!` re-open names it `__Table__`, while a `#[default_impl]`/`#[prefix]` registration
/// names it `__Components__`. The name is a reserved identifier the CGP macros emit, so it is a
/// reliable, `DefId`-independent discriminator.
fn orphan_trigger(tcx: TyCtxt<'_>, impl_did: DefId) -> OrphanTrigger {
    let is_table = tcx.generics_of(impl_did).own_params.iter().any(|param| {
        matches!(param.kind, ty::GenericParamDefKind::Type { .. })
            && param.name.as_str() == "__Table__"
    });
    if is_table {
        OrphanTrigger::Reopen
    } else {
        OrphanTrigger::Register
    }
}

/// Whether two spans cover overlapping source ranges, ignoring `SyntaxContext` — the macro-generated
/// impl and the coherence caret carry different contexts, so the match compares ranges, as the
/// duplicate-key conflict classifier's `same_range` does.
fn range_overlaps(a: Span, b: Span) -> bool {
    a.lo() <= b.hi() && b.lo() <= a.hi()
}
