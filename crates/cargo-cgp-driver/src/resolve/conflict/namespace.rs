//! Classifying an `E0119` on a namespace's own lookup trait.
//!
//! An entry *inside a namespace* conflicts on that namespace's own lookup trait — the `E0119`
//! reads `conflicting implementations of trait `MyNamespace<_>` for type `PathCons<…>`` — so
//! neither `DelegateComponent` nor `IsProviderFor` appears in the message and the pair routing
//! cannot reach it. This classifier recognizes the shape by the impls at the carets instead: a
//! local impl of a [namespace lookup trait](crate::resolve::cgp_item::is_namespace_lookup_trait),
//! whose `Self` is the entry's key. The recovered conflict reuses the same [`WiringConflict`]
//! shapes as a context-table collision, with the namespace trait standing in as the wired-on
//! subject.
//!
//! Three sources emit such an impl, and all three land here: a `cgp_namespace!` body entry, a
//! provider's `#[default_impl(… in Namespace)]` registration, and the inheritance blanket a
//! `new Child: Parent` header emits. That last one is why the collision is often not the duplicate
//! it looks like: the blanket forwards *every* key the parent answers, so binding a key the parent
//! already registers collides with it. When the parent resolves that key to a `RedirectLookup` —
//! the shape a component's `#[prefix(...)]` produces — the real mistake is that the key is
//! addressed by its path, and [`redirect_conflict`](super::redirect_conflict) recovers that path so
//! the message names the entry to write instead.

use cargo_cgp_error_processing::WiringConflict;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::resolve::cgp_item::is_namespace_lookup_trait;
use crate::resolve::conflict::{
    ConflictSide, describe_key, impl_delegate_type, maps_one_key, overlap_conflict,
    redirect_conflict, render_lookup_trait, same_range,
};

/// Classify an `E0119` on a namespace lookup trait as a duplicate-entry conflict, or `None` when
/// no namespace-trait impl sits at the caret (leaving the diagnostic to the ordinary fallback).
/// The shapes mirror [`build_conflict`](super::build_conflict): of two entries over one key, both
/// redirecting is a duplicate redirect (naming both targets), one redirecting — written, or
/// inherited through a parent namespace — is a redirect collision, and two plain entries are a
/// duplicate; two distinct keys are an overlap.
pub(crate) fn classify_namespace_conflict(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    label_spans: &[Span],
) -> Option<WiringConflict> {
    let mut at_caret = namespace_entries_at(tcx, primary_span).into_iter();
    let first_at_caret = at_caret.next()?;
    let first = label_spans
        .iter()
        .filter(|&&span| !same_range(span, primary_span))
        .find_map(|&span| namespace_entries_at(tcx, span).into_iter().next());
    // Two entries one macro emits from one attribute site, such as two `#[prefix]` attributes on
    // one component, carry the same span, so rustc puts both carets on it: the first entry there is
    // the first implementation, and the second the conflicting one.
    let (conflicting, first) = match first {
        Some(first) => (first_at_caret, Some(first)),
        None => match at_caret.next() {
            Some(second) => (second, Some(first_at_caret)),
            None => (first_at_caret, None),
        },
    };

    let namespace = render_lookup_trait(tcx, conflicting.impl_did);
    let key = describe_key(tcx, conflicting.key, conflicting.impl_did)?;
    // Without the first entry the tool cannot say what the key collides with (rustc names an
    // upstream impl only in a note), so the diagnostic keeps rustc's wording.
    let first = first?;
    // An entry can also collide with the namespace's own *inheritance* blanket, whose `Delegate`
    // is a projection rather than a written redirect; `redirect_conflict` resolves what the parent
    // maps the colliding key to.
    if maps_one_key(tcx, &first, &conflicting)
        && let Some(conflict) = redirect_conflict(tcx, &namespace, &first, &conflicting)
    {
        return Some(conflict);
    }
    let first_key = describe_key(tcx, first.key, first.impl_did)?;
    if first_key == key {
        Some(WiringConflict::Duplicate {
            context: namespace,
            key,
        })
    } else {
        overlap_conflict(tcx, namespace, (&first, first_key), (&conflicting, key))
    }
}

/// The namespace entries whose namespace-trait impls sit at `span`, in definition order — matched
/// by source range against each impl's def-span, exactly as the `DelegateComponent` classifier
/// matches its carets. Empty when no local namespace-trait impl is there.
fn namespace_entries_at<'tcx>(tcx: TyCtxt<'tcx>, span: Span) -> Vec<ConflictSide<'tcx>> {
    let mut entries = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_did = local.to_def_id();
        if !same_range(tcx.def_span(impl_did), span) {
            continue;
        }
        let trait_did = tcx.impl_trait_ref(impl_did).skip_binder().def_id;
        if !is_namespace_lookup_trait(tcx, trait_did) {
            continue;
        }
        entries.push(ConflictSide {
            impl_did,
            key: tcx.type_of(impl_did).instantiate_identity().skip_norm_wip(),
            delegate: impl_delegate_type(tcx, impl_did),
        });
    }
    entries
}
