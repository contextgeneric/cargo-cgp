//! Classifying an `E0119` on the user's own `cgp_namespace!` trait.
//!
//! A duplicate entry *inside a namespace* conflicts on the namespace's own lookup trait — the
//! `E0119` reads `conflicting implementations of trait `MyNamespace<_>` for type `PathCons<…>`` —
//! so neither `DelegateComponent` nor `IsProviderFor` appears in the message and the pair routing
//! cannot reach it. This classifier recognizes the shape by the impls at the carets instead: a
//! local impl of a [namespace lookup trait](crate::resolve::cgp_item::is_namespace_lookup_trait)
//! whose `Self` is the entry's path key. The recovered conflict reuses the same
//! [`WiringConflict`] shapes as a context-table collision, with the namespace trait standing in
//! as the wired-on subject.

use cargo_cgp_error_processing::{WiringConflict, WiringKey};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::is_namespace_lookup_trait;
use crate::resolve::conflict::{
    impl_delegate_type, redirect_path, render_path, render_provider, same_range,
};

/// One `cgp_namespace!` entry, read off its namespace-trait impl: the namespace trait it
/// registers into, the path key (the impl's `Self`), and the `Delegate` it maps the key to.
struct NamespaceEntry<'tcx> {
    trait_did: DefId,
    path: Ty<'tcx>,
    delegate: Option<Ty<'tcx>>,
}

/// Classify an `E0119` on a user namespace trait as a duplicate-entry conflict, or `None` when no
/// namespace-trait impl sits at the caret (leaving the diagnostic to the ordinary fallback). The
/// shapes mirror [`build_conflict`](super::build_conflict): both entries redirecting is a
/// duplicate redirect (naming both targets), one redirecting is a redirect collision, and two
/// plain entries are a duplicate or an overlap by whether their rendered paths agree.
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

    let namespace = tcx.item_name(conflicting.trait_did).to_string();
    let key = WiringKey::Path(render_path(tcx, conflicting.path)?);

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
            let first_key = WiringKey::Path(render_path(tcx, first.path)?);
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

/// The `cgp_namespace!` entry whose namespace-trait impl sits at `span` — matched by source range
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
            trait_did,
            path: tcx.type_of(impl_did).instantiate_identity().skip_norm_wip(),
            delegate: impl_delegate_type(tcx, impl_did),
        });
    }
    None
}
