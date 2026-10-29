//! Detecting a provider-trait impl written without `#[cgp_provider]`.
//!
//! A provider trait has `IsProviderFor<Component, Context, Params>` as a supertrait, so the impl of
//! the provider trait for a provider struct must come with an impl of `IsProviderFor` for the same
//! struct and component, which `#[cgp_provider]` (and `#[cgp_impl]`, which lowers to it) writes. An
//! impl written by hand has none, and the compiler rejects it on the supertrait bound at the impl's
//! `Self` type. This module recognizes that shape off the compiler, never from error text: the
//! failing span is the `Self` type of a local impl whose trait is a CGP provider trait the user
//! named, whose `Self` is a struct, enum, or union, and for which no `IsProviderFor` impl of that
//! `Self` and component exists.

use cargo_cgp_error_processing::ProviderImplWithoutMacro;
use rustc_hir::def::DefKind;
use rustc_hir::{ItemKind, QPath, TyKind};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, IS_PROVIDER_FOR_TRAIT};
use crate::resolve::query::{find_cgp_trait, provider_blanket_marker};

/// Recognize the `E0277` whose caret is `primary_span` as the missing `IsProviderFor` of a
/// provider-trait impl written without its macro, or `None` when it is not one.
pub fn detect_provider_impl_without_macro(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<ProviderImplWithoutMacro> {
    let is_provider_for = find_cgp_trait(tcx, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)?;
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let ItemKind::Impl(imp) = tcx.hir_expect_item(local).kind else {
            continue;
        };
        if imp.self_ty.span.lo() != primary_span.lo() || imp.self_ty.span.hi() != primary_span.hi()
        {
            continue;
        }
        let header = imp.of_trait?;
        if header.trait_ref.path.span.from_expansion() {
            continue;
        }
        let trait_did = header.trait_ref.path.res.opt_def_id()?;
        let marker = provider_blanket_marker(tcx, trait_did)?;
        let ty::Adt(marker_def, _) = marker.kind() else {
            continue;
        };
        let TyKind::Path(QPath::Resolved(_, self_path)) = imp.self_ty.kind else {
            continue;
        };
        let self_did = self_path.res.opt_def_id()?;
        if !matches!(
            tcx.def_kind(self_did),
            DefKind::Struct | DefKind::Enum | DefKind::Union
        ) {
            continue;
        }
        let has_marker_impl = tcx.all_impls(is_provider_for).any(|impl_did| {
            let impl_ref = tcx.impl_trait_ref(impl_did).skip_binder();
            matches!(impl_ref.self_ty().kind(), ty::Adt(def, _) if def.did() == self_did)
                && matches!(impl_ref.args.type_at(1).kind(), ty::Adt(def, _) if def.did() == marker_def.did())
        });
        if has_marker_impl {
            continue;
        }
        return Some(ProviderImplWithoutMacro {
            provider: tcx.item_name(self_did).to_string(),
            provider_trait: tcx.item_name(trait_did).to_string(),
        });
    }
    None
}
