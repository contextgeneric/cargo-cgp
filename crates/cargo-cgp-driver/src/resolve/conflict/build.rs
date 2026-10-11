//! Wording the recovered conflicting impls into a [`WiringConflict`].

use cargo_cgp_error_processing::{WiringConflict, WiringKey};
use rustc_middle::ty::TyCtxt;

use crate::resolve::conflict::{
    ConflictSide, DelegateImpl, EntryTarget, component_params, describe_key, redirect_path,
};

/// Word the recovered impls into a [`WiringConflict`], picking the shape from how the two entries
/// relate: a redirect (one or both map the key to a `RedirectLookup`), or a plain duplicate or
/// overlap otherwise. `None` when a key cannot be rendered to a surface form.
pub(crate) fn build_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    conflicting: &DelegateImpl<'tcx>,
    first: Option<&DelegateImpl<'tcx>>,
) -> Option<WiringConflict> {
    let context = tcx
        .erase_and_anonymize_regions(conflicting.self_ty)
        .to_string();
    let conflicting_side = ConflictSide {
        impl_did: conflicting.impl_did,
        key: conflicting.key,
        delegate: conflicting.delegate,
    };
    // Without the first entry, a redirect has no provider to name, so only a duplicate is worded.
    let Some(first) = first else {
        if conflicting
            .delegate
            .and_then(|d| redirect_path(tcx, d))
            .is_some()
        {
            return None;
        }
        let key = describe_key(tcx, conflicting.key, conflicting.impl_did)?;
        return Some(WiringConflict::Duplicate { context, key });
    };
    let first_side = ConflictSide {
        impl_did: first.impl_did,
        key: first.key,
        delegate: first.delegate,
    };
    if let Some(conflict) = redirect_conflict(tcx, &context, &first_side, &conflicting_side) {
        return Some(conflict);
    }

    let key = describe_key(tcx, conflicting.key, conflicting.impl_did)?;
    let first_key = describe_key(tcx, first.key, first.impl_did)?;
    Some(match (&first_key, &key) {
        // Two blanket forwardings, each over every key — a context joining more than one
        // namespace (`namespace` desugars to a bare-key `for` loop, so the two are the same shape).
        (WiringKey::Blanket(first_ns), WiringKey::Blanket(second_ns)) => {
            WiringConflict::MultipleNamespaces {
                context,
                first: first_ns.clone(),
                second: second_ns.clone(),
            }
        }
        _ if first_key == key => WiringConflict::Duplicate { context, key },
        _ => WiringConflict::Overlap {
            context,
            conflicting: key,
            first: first_key,
        },
    })
}

/// Recognize a collision in which at least one entry redirects the key: both redirecting is a
/// duplicate redirect (naming both targets), one is a redirect collision whose fix names the other
/// entry's provider. A blanket forwarding (a namespace join, or a namespace's inheritance blanket)
/// redirects when it maps the other entry's concrete key to a `RedirectLookup`, and the message then
/// names that concrete key. `None` when neither entry redirects. `subject` is the context or the
/// namespace the collision is reported on.
pub(crate) fn redirect_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    subject: &str,
    first: &ConflictSide<'tcx>,
    conflicting: &ConflictSide<'tcx>,
) -> Option<WiringConflict> {
    let first_target = first.target(tcx, conflicting.key);
    let conflicting_target = conflicting.target(tcx, first.key);
    // The key both entries map, as written: the concrete side's when one is a blanket.
    let named = if conflicting.is_blanket() {
        first
    } else {
        conflicting
    };
    let key_ty = named.key;
    let key = || describe_key(tcx, key_ty, named.impl_did);
    let context = subject.to_owned();
    match (first_target, conflicting_target) {
        (EntryTarget::Redirect(first_path), EntryTarget::Redirect(second_path)) => {
            Some(WiringConflict::DuplicateRedirect {
                context,
                key: key()?,
                first_path,
                second_path,
            })
        }
        (EntryTarget::Redirect(path), EntryTarget::Provider(provider))
        | (EntryTarget::Provider(provider), EntryTarget::Redirect(path)) => {
            Some(WiringConflict::Redirect {
                context,
                key: key()?,
                path,
                provider,
                params: component_params(tcx, key_ty),
            })
        }
        (EntryTarget::Provider(_), EntryTarget::Provider(_)) => None,
    }
}
