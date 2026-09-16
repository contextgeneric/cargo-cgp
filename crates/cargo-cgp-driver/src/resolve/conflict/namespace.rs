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
//! addressed by its path, and [`inherited_redirect_conflict`] recovers that path so the message
//! names the entry to write instead.

use cargo_cgp_error_processing::{WiringConflict, WiringKey};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::is_namespace_lookup_trait;
use crate::resolve::conflict::{
    describe_key, impl_delegate_type, namespace_redirect, redirect_path, render_lookup_trait,
    render_provider, same_range,
};

/// One namespace entry, read off its namespace-trait impl: the impl itself, the key it registers
/// (the impl's `Self` — an `@`-path, a component marker, a per-type key, or the generic parameter
/// of an inheritance blanket), and the `Delegate` it maps that key to.
struct NamespaceEntry<'tcx> {
    impl_did: DefId,
    key: Ty<'tcx>,
    delegate: Option<Ty<'tcx>>,
}

/// Classify an `E0119` on a namespace lookup trait as a duplicate-entry conflict, or `None` when
/// no namespace-trait impl sits at the caret (leaving the diagnostic to the ordinary fallback).
/// The shapes mirror [`build_conflict`](super::build_conflict): both entries redirecting is a
/// duplicate redirect (naming both targets), one redirecting — written, or inherited through a
/// parent namespace — is a redirect collision, and two plain entries are a duplicate or an overlap
/// by whether their rendered keys agree.
pub(crate) fn classify_namespace_conflict(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    label_spans: &[Span],
) -> Option<WiringConflict> {
    let conflicting = namespace_entry_at(tcx, primary_span)?;
    let first = label_spans
        .iter()
        .filter(|&&span| !same_range(span, primary_span))
        .find_map(|&span| namespace_entry_at(tcx, span));

    let namespace = render_lookup_trait(tcx, conflicting.impl_did);
    let key = describe_key(tcx, conflicting.key, conflicting.impl_did)?;

    let conflicting_redirect = conflicting.delegate.and_then(|d| redirect_path(tcx, d));
    let first_redirect = first
        .as_ref()
        .and_then(|f| f.delegate)
        .and_then(|d| redirect_path(tcx, d));

    match (first_redirect, conflicting_redirect) {
        (Some(first_path), Some(second_path)) => Some(WiringConflict::DuplicateRedirect {
            context: namespace,
            key,
            first_path,
            second_path,
        }),
        (Some(path), None) => Some(WiringConflict::Redirect {
            context: namespace,
            key,
            path,
            provider: render_provider(tcx, conflicting.delegate?),
        }),
        (None, Some(path)) => Some(WiringConflict::Redirect {
            context: namespace,
            key,
            path,
            provider: render_provider(tcx, first?.delegate?),
        }),
        (None, None) => {
            let Some(first) = first else {
                return Some(WiringConflict::Duplicate {
                    context: namespace,
                    key,
                });
            };
            let first_key = describe_key(tcx, first.key, first.impl_did)?;

            // An entry can also collide with the namespace's own *inheritance* blanket, whose
            // `Delegate` is a projection rather than a written redirect. When the parent resolves
            // the colliding key to a `RedirectLookup`, that is the whole mistake — the key is
            // addressed by a path — so recover it and say so.
            if let Some(redirect) =
                inherited_redirect_conflict(tcx, &namespace, &conflicting, &first, &key, &first_key)
            {
                return Some(redirect);
            }

            Some(if first_key == key {
                WiringConflict::Duplicate {
                    context: namespace,
                    key,
                }
            } else {
                WiringConflict::Overlap {
                    context: namespace,
                    conflicting: key,
                    first: first_key,
                }
            })
        }
    }
}

/// Recognize a namespace entry colliding with an *inheritance blanket* that redirects the same key:
/// one of the two entries is the blanket a `new Child: Parent` header emits, the other a concrete
/// key the parent maps to a `RedirectLookup` — the shape a component's
/// [`#[prefix(...)]`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/attributes/prefix.md)
/// registration produces, where the key is addressed by its path rather than by its bare marker.
/// The message then names that path, and its `help` the entry to write instead. `None` when
/// neither entry is a blanket, or the parent does not redirect the concrete key.
fn inherited_redirect_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    namespace: &str,
    conflicting: &NamespaceEntry<'tcx>,
    first: &NamespaceEntry<'tcx>,
    key: &WiringKey,
    first_key: &WiringKey,
) -> Option<WiringConflict> {
    let is_blanket = |k: &WiringKey| matches!(k, WiringKey::Blanket(_));
    let (blanket, concrete) = match (is_blanket(key), is_blanket(first_key)) {
        (true, false) => (conflicting, first),
        (false, true) => (first, conflicting),
        _ => return None,
    };
    let path = namespace_redirect(tcx, blanket.impl_did, blanket.key, concrete.key)?;
    Some(WiringConflict::Redirect {
        context: namespace.to_owned(),
        key: describe_key(tcx, concrete.key, concrete.impl_did)?,
        path,
        provider: render_provider(tcx, concrete.delegate?),
    })
}

/// The namespace entry whose namespace-trait impl sits at `span` — matched by source range
/// against the impl's def-span, exactly as the `DelegateComponent` classifier matches its carets —
/// or `None` when no local namespace-trait impl is there.
fn namespace_entry_at<'tcx>(tcx: TyCtxt<'tcx>, span: Span) -> Option<NamespaceEntry<'tcx>> {
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
        return Some(NamespaceEntry {
            impl_did,
            key: tcx.type_of(impl_did).instantiate_identity().skip_norm_wip(),
            delegate: impl_delegate_type(tcx, impl_did),
        });
    }
    None
}
