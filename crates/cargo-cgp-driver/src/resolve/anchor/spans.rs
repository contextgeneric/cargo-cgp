//! Finding the local items a diagnostic's spans land on.
//!
//! Every span-matching anchor starts from these lookups: the trait impls a failure can surface
//! inside, and the struct/enum definitions a use-site failure's spans touch.

use rustc_hir::ItemKind;
use rustc_hir::def::DefKind;
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::cgp_item::is_provider_struct;

/// The local trait-impl blocks (`impl Trait for Ty { … }`) whose source span contains one of the
/// diagnostic's spans — the impls a wiring failure can surface inside, whether the caret lands on
/// the impl header or deep in a method body. A trait impl (not an inherent one) is required
/// because the recovery reads the impl's trait supertraits. The *full* item span is taken from HIR
/// rather than `def_span`, which for an impl covers only the header and would miss a caret inside
/// the body (a forwarding `self.method(..)` call, say).
pub(crate) fn enclosing_trait_impls(tcx: TyCtxt<'_>, spans: &[Span]) -> Vec<DefId> {
    let mut impls = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_span = tcx.hir_expect_item(local).span;
        if spans.iter().any(|&span| impl_span.contains(span)) {
            impls.push(local.to_def_id());
        }
    }
    impls
}

/// The candidate context types of a use-site failure: every local struct or enum whose definition
/// span contains one of the diagnostic's spans — for an `E0599` method error that includes the
/// "method not found for this struct" span on the receiver's type. Each ADT is returned with
/// identity arguments (so a generic context keeps its generic form). A provider struct that merely
/// shares a span — rustc names it in a "required for … to implement …" note — is excluded, since a
/// provider is never a context; the caller then picks the candidate that actually wires a failing
/// component.
pub(crate) fn context_candidates_from_spans<'tcx>(
    tcx: TyCtxt<'tcx>,
    spans: &[Span],
) -> Vec<Ty<'tcx>> {
    let mut candidates = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        let did = local.to_def_id();
        if !matches!(tcx.def_kind(did), DefKind::Struct | DefKind::Enum) {
            continue;
        }
        let def_span = tcx.def_span(did);
        if spans.iter().any(|&span| def_span.contains(span)) {
            let ty = tcx.type_of(did).instantiate_identity().skip_norm_wip();
            // A provider struct is never a context; exclude it so an anchor without a wiring check
            // (e.g. `resolve_use_site_consumer`) does not mis-recover it as the failing context.
            if !is_provider_struct(tcx, ty) {
                candidates.push(ty);
            }
        }
    }
    candidates
}

/// The source span of an impl's `Self` type, e.g. the `Rectangle` in
/// `impl __CheckRectangle<..> for Rectangle {}` — which the check macro re-spans onto the
/// `check_components!` entry, so it matches the failing diagnostic's primary span.
pub(crate) fn impl_self_ty_span(tcx: TyCtxt<'_>, impl_did: DefId) -> Option<Span> {
    let local = impl_did.as_local()?;
    match tcx.hir_expect_item(local).kind {
        ItemKind::Impl(imp) => Some(imp.self_ty.span),
        _ => None,
    }
}
