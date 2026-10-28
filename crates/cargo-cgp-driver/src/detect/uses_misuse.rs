//! Detecting a provider trait named in `#[uses(…)]`.
//!
//! `#[uses(AreaCalculator)]` on a `#[cgp_fn]` or `#[cgp_impl]` writes the bound `Self:
//! AreaCalculator` on the generated impl, but a provider trait takes the context as its first
//! argument, which `#[uses]` does not supply, so the compiler reports `E0107`, missing generics for
//! trait `AreaCalculator`, and suggests adding the generated `__Context__`. A dependency on a
//! component names its consumer trait (`CanCalculateArea`), so this module recovers the provider
//! trait written and the consumer trait it pairs with, filling the rustc-free
//! [`ProviderTraitInUses`] the emitter words into a `[CGP-E037]` header.
//!
//! The `E0107` arrives while the compiler lowers the impl's `where` clause, so the bound is read from
//! the HIR, and the search for the consumer trait skips the trait the impl implements, whose blanket
//! impl may be the one being lowered.

use cargo_cgp_error_processing::ProviderTraitInUses;
use rustc_hir::def::DefKind;
use rustc_hir::{GenericArg, GenericBound, ItemKind, QPath, TyKind, WherePredicateKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::CGP_IMPL_CONTEXT_PARAM;
use crate::detect::unimported::has_context_param;
use crate::resolve::cgp_item::{consumer_provider_trait, is_provider_trait};

/// Recognize an `E0107` at `primary_span` as a provider trait named in `#[uses(…)]`: the caret is
/// the trait name of a `where` bound on the context (`Self` or `__Context__`) of an impl a CGP macro
/// generated, the trait is a provider trait written without arguments, and a consumer trait pairs
/// with it. `None` otherwise.
pub fn detect_provider_trait_in_uses(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<ProviderTraitInUses> {
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true })
            || !has_context_param(tcx, local)
        {
            continue;
        }
        let ItemKind::Impl(imp) = tcx.hir_expect_item(local).kind else {
            continue;
        };
        let own_trait = imp
            .of_trait
            .and_then(|header| header.trait_ref.path.res.opt_def_id());
        for predicate in imp.generics.predicates {
            let WherePredicateKind::BoundPredicate(bound) = predicate.kind else {
                continue;
            };
            if !bounds_context(bound.bounded_ty) {
                continue;
            }
            for generic_bound in bound.bounds {
                let GenericBound::Trait(poly) = generic_bound else {
                    continue;
                };
                let path = poly.trait_ref.path;
                let Some(segment) = path.segments.last() else {
                    continue;
                };
                if !segment.ident.span.overlaps(primary_span) {
                    continue;
                }
                let written_args = segment.args.is_some_and(|args| {
                    args.args
                        .iter()
                        .any(|arg| matches!(arg, GenericArg::Type(_)))
                });
                let Some(provider_did) = path.res.opt_def_id() else {
                    continue;
                };
                if written_args || !is_provider_trait(tcx, provider_did) {
                    continue;
                }
                let consumer_did = consumer_of(tcx, provider_did, own_trait)?;
                return Some(ProviderTraitInUses {
                    provider: tcx.item_name(provider_did).to_string(),
                    consumer: tcx.item_name(consumer_did).to_string(),
                });
            }
        }
    }
    None
}

/// Whether a `where` bound's bounded type is the generated context: `Self` in a `#[cgp_fn]` impl, or
/// the `__Context__` `#[cgp_impl]` writes for it.
fn bounds_context(bounded: &rustc_hir::Ty<'_>) -> bool {
    let TyKind::Path(QPath::Resolved(None, path)) = bounded.kind else {
        return false;
    };
    matches!(path.segments, [segment]
        if segment.ident.name.as_str() == "Self"
            || segment.ident.name.as_str() == CGP_IMPL_CONTEXT_PARAM)
}

/// The consumer trait whose blanket impl routes to `provider_did` ([`consumer_provider_trait`]),
/// leaving out `skip`, the trait of the impl being lowered.
fn consumer_of(tcx: TyCtxt<'_>, provider_did: DefId, skip: Option<DefId>) -> Option<DefId> {
    tcx.all_traits_including_private()
        .filter(|&trait_did| Some(trait_did) != skip)
        .find(|&trait_did| consumer_provider_trait(tcx, trait_did) == Some(provider_did))
}
