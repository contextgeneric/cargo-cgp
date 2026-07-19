//! Matching an `E0119` caret to the conflicting wiring impls and routing the pair.

use cargo_cgp_error_processing::WiringConflict;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT};
use crate::resolve::cgp_item::{find_cgp_trait, is_cgp_item};
use crate::resolve::conflict::{build_conflict, classify_namespace_conflict, local_delegate_impls};

/// Which trait an `E0119` conflict is about, read from the diagnostic's own message — the only
/// thing that tells the `DelegateComponent` half of the pair from its `IsProviderFor` half, since
/// both carry the same caret. The classifier still verifies the genuine CGP impls sit at that
/// caret before acting, so the message is used only to route within a confirmed pair. An `E0119`
/// naming neither trait can still be a wiring conflict — a duplicate `cgp_namespace!` entry
/// conflicts on the *user's own* namespace trait — so [`Other`](ConflictTrait::Other) routes to
/// the namespace classifier.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConflictTrait {
    Delegate,
    IsProviderFor,
    Other,
}

/// What to do with an `E0119` recognized as a duplicate-key wiring conflict.
pub enum ConflictAction {
    /// Drop this diagnostic entirely — it is the redundant `IsProviderFor` half of the pair.
    Suppress,
    /// Rewrite this diagnostic's header from the recovered conflict.
    Rewrite(WiringConflict),
}

/// Whether two spans cover the same source range, ignoring `SyntaxContext`. The two halves of the
/// `E0119` pair are re-spanned onto the same entry token but carry different contexts, so matching
/// a caret to an impl must compare ranges, not whole spans.
pub(crate) fn same_range(a: Span, b: Span) -> bool {
    a.lo() == b.lo() && a.hi() == b.hi()
}

/// Classify an `E0119` at `primary_span` as a duplicate-key wiring conflict, or return `None` if
/// no CGP wiring conflict sits at that caret (so the caller leaves the diagnostic to the ordinary
/// fallback). `variant` routes the recognized shapes: the `IsProviderFor` half of a generated
/// pair is dropped (when its companion conflict is confirmed to be reported alongside), a
/// `DelegateComponent` half is rewritten from the conflicting entries, and any other trait is
/// tried as a `cgp_namespace!` conflict on the user's own namespace trait. `label_spans` are the
/// diagnostic's labelled spans, searched (minus the primary) for the "first implementation here"
/// impl.
pub fn classify_wiring_conflict(
    tcx: TyCtxt<'_>,
    variant: ConflictTrait,
    primary_span: Span,
    label_spans: &[Span],
) -> Option<ConflictAction> {
    // An `IsProviderFor` conflict is always the redundant half of a generated pair — the macro
    // emits the `IsProviderFor` impl alongside a `DelegateComponent` entry impl or a provider
    // trait impl, and the paired impls conflict exactly when the `IsProviderFor` copies do — so it
    // is dropped whenever the companion pair is confirmed, whichever trait it is on.
    if variant == ConflictTrait::IsProviderFor {
        return suppressible_is_provider_for(tcx, primary_span, label_spans)
            .then_some(ConflictAction::Suppress);
    }

    if variant == ConflictTrait::Other {
        // A duplicate `cgp_namespace!` entry conflicts on the user's own namespace trait, which
        // the message names instead of `DelegateComponent`.
        return classify_namespace_conflict(tcx, primary_span, label_spans)
            .map(ConflictAction::Rewrite);
    }

    let delegate_did = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;
    let impls = local_delegate_impls(tcx, delegate_did);

    // The conflicting entry is the impl whose def-span the caret sits on. Its presence is what
    // confirms this `E0119` is a real `DelegateComponent` duplicate. Spans are matched by source
    // range, not `==`: the `IsProviderFor` and `DelegateComponent` halves of a pair carry the same
    // caret range but under different `SyntaxContext`s, so an exact comparison would miss the
    // match for one half.
    let conflicting = impls
        .iter()
        .find(|i| same_range(i.def_span, primary_span))?;

    // The "first implementation here" impl is at whichever other labelled span matches an impl.
    let first = label_spans
        .iter()
        .filter(|&&sp| !same_range(sp, primary_span))
        .find_map(|&sp| impls.iter().find(|i| same_range(i.def_span, sp)));

    Some(ConflictAction::Rewrite(build_conflict(
        tcx,
        conflicting,
        first,
    )?))
}

/// Whether an `E0119` on `IsProviderFor` impls is safe to drop as the redundant half of a
/// generated pair. Dropping is safe only when the *companion* conflict — the one on the impls the
/// same macro invocations generated the `IsProviderFor` copies alongside — is itself reported, or
/// the user would face a failing build with no error at all. That companion is confirmed
/// structurally: the caret must sit on a genuine local `IsProviderFor` impl, and the two colliding
/// sites (the caret and a labelled span) must each also carry a local impl of one *common* other
/// trait — `DelegateComponent` for a duplicate wiring entry, or the provider trait for a duplicate
/// provider definition — whose own `E0119` rustc then reports. Two `IsProviderFor` impls with *no*
/// common companion trait (a delegation entry colliding with a provider impl, say) are not
/// suppressed, since no companion conflict would surface the mistake.
fn suppressible_is_provider_for(tcx: TyCtxt<'_>, primary_span: Span, label_spans: &[Span]) -> bool {
    let (at_primary_is_provider_for, at_primary): (Vec<DefId>, Vec<DefId>) =
        local_trait_impls_at(tcx, primary_span)
            .into_iter()
            .partition(|&did| is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE));
    if at_primary_is_provider_for.is_empty() || at_primary.is_empty() {
        return false;
    }
    label_spans
        .iter()
        .filter(|&&span| !same_range(span, primary_span))
        .any(|&span| {
            local_trait_impls_at(tcx, span)
                .iter()
                .any(|did| at_primary.contains(did))
        })
}

/// The traits with a local impl whose def-span covers the same source range as `span` — the
/// impls a conflict caret can stand for, since rustc aims an `E0119` at the conflicting impl's
/// def-span and the CGP macros re-span every impl of one entry onto the same token.
fn local_trait_impls_at(tcx: TyCtxt<'_>, span: Span) -> Vec<DefId> {
    let mut traits = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let did = local.to_def_id();
        if !same_range(tcx.def_span(did), span) {
            continue;
        }
        let trait_did = tcx.impl_trait_ref(did).skip_binder().def_id;
        if !traits.contains(&trait_did) {
            traits.push(trait_did);
        }
    }
    traits
}
