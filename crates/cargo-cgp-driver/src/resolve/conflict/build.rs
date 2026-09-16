//! Wording the recovered conflicting impls into a [`WiringConflict`].

use cargo_cgp_error_processing::{WiringConflict, WiringKey};
use rustc_middle::ty::TyCtxt;

use crate::resolve::conflict::{
    DelegateImpl, describe_key, namespace_redirect, redirect_path, render_provider,
};

/// Word the recovered impls into a [`WiringConflict`], picking the shape from how the two entries
/// relate: a redirect (one or both `Delegate`s are `RedirectLookup`), or a plain
/// duplicate/overlap otherwise. `None` when a key cannot be rendered to a surface form.
pub(crate) fn build_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    conflicting: &DelegateImpl<'tcx>,
    first: Option<&DelegateImpl<'tcx>>,
) -> Option<WiringConflict> {
    let context = tcx
        .erase_and_anonymize_regions(conflicting.self_ty)
        .to_string();
    let key = describe_key(tcx, conflicting.key, conflicting.impl_did)?;

    let conflicting_redirect = conflicting.delegate.and_then(|d| redirect_path(tcx, d));
    let first_redirect = first
        .and_then(|f| f.delegate)
        .and_then(|d| redirect_path(tcx, d));

    match (first_redirect, conflicting_redirect) {
        // Both entries redirect the same key — a duplicate redirect (the two targets may differ).
        (Some(first_path), Some(second_path)) => {
            return Some(WiringConflict::DuplicateRedirect {
                context,
                key,
                first_path,
                second_path,
            });
        }
        // One entry redirects the key while the other sets it directly — a redirect collision. The
        // provider named in the fix comes from the *direct* entry (the non-redirect one).
        (Some(path), None) => {
            let provider = render_provider(tcx, conflicting.delegate?);
            return Some(WiringConflict::Redirect {
                context,
                key,
                path,
                provider,
            });
        }
        (None, Some(path)) => {
            let provider = render_provider(tcx, first?.delegate?);
            return Some(WiringConflict::Redirect {
                context,
                key,
                path,
                provider,
            });
        }
        (None, None) => {}
    }

    let Some(first) = first else {
        return Some(WiringConflict::Duplicate { context, key });
    };
    let first_key = describe_key(tcx, first.key, first.impl_did)?;

    // A direct wiring can also collide with a *namespace* that redirects the same key — the blanket
    // forwarding's `Delegate` for that concrete key normalizes to a `RedirectLookup`. Recover that
    // here, so it reads as a redirect collision rather than a bare overlap.
    if let Some(redirect) =
        namespace_redirect_conflict(tcx, &context, conflicting, first, &key, &first_key)
    {
        return Some(redirect);
    }

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

/// Recognize a direct wiring colliding with a *namespace* that redirects the same key: one of the
/// two entries is a blanket namespace forwarding, the other a concrete key the namespace maps to a
/// `RedirectLookup`. The message then reads as a redirect collision — wire the direct entry's
/// provider under the redirected path — rather than a bare overlap. `None` when neither entry is a
/// blanket, or the namespace does not redirect the concrete key.
fn namespace_redirect_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: &str,
    conflicting: &DelegateImpl<'tcx>,
    first: &DelegateImpl<'tcx>,
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
        context: context.to_owned(),
        key: describe_key(tcx, concrete.key, concrete.impl_did)?,
        path,
        provider: render_provider(tcx, concrete.delegate?),
    })
}
