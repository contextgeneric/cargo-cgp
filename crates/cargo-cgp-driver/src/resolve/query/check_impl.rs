//! Reading a `check_components!` check impl: the check trait's assertion and the impl's caret.

use rustc_hir::ItemKind;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE};
use crate::resolve::query::is_cgp_item;

/// The `CanUseComponent<..>` supertrait clause of `trait_did`, if it carries one — the marker
/// of a `check_components!` check trait. Anchored by DefId to `cgp_component`.
pub fn can_use_component_supertrait(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<ty::Clause<'_>> {
    for &(clause, _) in tcx.explicit_super_clauses_of(trait_did).skip_binder() {
        if let Some(tp) = clause.as_trait_clause()
            && is_cgp_item(
                tcx,
                tp.def_id(),
                CAN_USE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        {
            return Some(clause);
        }
    }
    None
}
/// The source span of an impl's `Self` type, e.g. the `Rectangle` in
/// `impl __CheckRectangle<..> for Rectangle {}` — which the check macro re-spans onto the
/// `check_components!` entry, so it matches the failing diagnostic's primary span.
pub fn impl_self_ty_span(tcx: TyCtxt<'_>, impl_did: DefId) -> Option<Span> {
    let local = impl_did.as_local()?;
    match tcx.hir_expect_item(local).kind {
        ItemKind::Impl(imp) => Some(imp.self_ty.span),
        _ => None,
    }
}

/// One check impl at a caret, as [`check_impls_at`] finds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckImpl<'tcx> {
    /// The check trait the impl implements.
    pub trait_did: DefId,
    /// The check impl.
    pub impl_did: DefId,
    /// The check trait's asserting supertrait, instantiated on the impl's trait ref (a
    /// `CanUseComponent<Marker, Params>` on the context, or an `IsProviderFor` on a provider).
    pub asserted: ty::Clause<'tcx>,
}

/// The check impls whose `Self` type sits at `primary_span`, lazily: for each trait whose
/// asserting supertrait `supertrait` reads (such as [`can_use_component_supertrait`]), each impl of
/// it whose `Self` type's span ([`impl_self_ty_span`]) is the caret, the check macro having
/// re-spanned that type onto the entry. Traits are scanned in the compiler's order, and a caller
/// that needs only the first match stops there.
pub fn check_impls_at<'tcx>(
    tcx: TyCtxt<'tcx>,
    primary_span: Span,
    supertrait: impl Fn(TyCtxt<'tcx>, DefId) -> Option<ty::Clause<'tcx>>,
) -> impl Iterator<Item = CheckImpl<'tcx>> {
    tcx.all_traits_including_private()
        .filter_map(move |trait_did| Some((trait_did, supertrait(tcx, trait_did)?)))
        .flat_map(move |(trait_did, super_clause)| {
            tcx.all_impls(trait_did)
                .filter(move |&impl_did| impl_self_ty_span(tcx, impl_did) == Some(primary_span))
                .map(move |impl_did| {
                    let trait_ref = tcx
                        .impl_trait_ref(impl_did)
                        .instantiate_identity()
                        .skip_norm_wip();
                    CheckImpl {
                        trait_did,
                        impl_did,
                        asserted: super_clause
                            .instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref)),
                    }
                })
        })
}
