//! Detecting a provider trait named in `#[uses(…)]`.
//!
//! `#[uses(AreaCalculator)]` on a `#[cgp_fn]` or `#[cgp_impl]` writes the bound `Self:
//! AreaCalculator` on the generated impl, but a provider trait takes the context as its first
//! argument, which `#[uses]` does not supply, so the compiler reports `E0107`, missing generics for
//! trait `AreaCalculator`, and suggests adding the generated `__Context__`. A generic component's
//! provider trait written with its own arguments (`AreaCalculator<Rectangle>`) gets the same
//! `E0107` as one argument too few, with a help that adds the component's parameter. A dependency
//! on a component names its consumer trait (`CanCalculateArea`), so this module recovers the
//! provider trait written and the consumer trait it pairs with, filling the rustc-free
//! [`ProviderTraitInUses`] the emitter words into a `[CGP-E037]` header.
//!
//! The `E0107` arrives while the compiler lowers the impl's `where` clause, so the bound is read
//! from the HIR, and the search for the consumer trait skips the trait the impl implements, whose
//! blanket impl may be the one being lowered.

use cargo_cgp_error_processing::ProviderTraitInUses;
use rustc_hir::def_id::DefId;
use rustc_hir::{
    GenericArg, GenericBound, Generics, ItemKind, Path, PathSegment, QPath, TyKind,
    WherePredicateKind,
};
use rustc_middle::ty::{GenericParamDefKind, TyCtxt};
use rustc_span::Span;

use super::cgp_impl_misuse::written_arguments;
use crate::config::CGP_IMPL_CONTEXT_PARAM;
use crate::resolve::query::{
    consumer_of_provider, has_context_param, is_provider_trait, local_trait_impls,
};

/// Recognize an `E0107` at `primary_span` as a provider trait named in `#[uses(…)]`: the caret is
/// the trait name of a `where` bound on the context (`Self` or `__Context__`) of an impl a CGP
/// macro generated, the trait is a provider trait written without its context, and a consumer trait
/// pairs with it. `None` otherwise.
pub fn detect_provider_trait_in_uses(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<ProviderTraitInUses> {
    for local in local_trait_impls(tcx) {
        if !has_context_param(tcx, local) {
            continue;
        }
        let ItemKind::Impl(imp) = tcx.hir_expect_item(local).kind else {
            continue;
        };
        let own_trait = imp
            .of_trait
            .and_then(|header| header.trait_ref.path.res.opt_def_id());
        for path in context_trait_bounds(imp.generics) {
            let Some(segment) = path.segments.last() else {
                continue;
            };
            if !segment.ident.span.overlaps(primary_span) {
                continue;
            }
            let Some(provider_did) = path.res.opt_def_id() else {
                continue;
            };
            if !is_provider_trait(tcx, provider_did)
                || !lacks_only_the_context(tcx, provider_did, segment)
            {
                continue;
            }
            let consumer_did = consumer_of_provider(tcx, provider_did, own_trait)?;
            return Some(ProviderTraitInUses {
                provider: tcx.item_name(provider_did).to_string(),
                consumer: tcx.item_name(consumer_did).to_string(),
                arguments: written_arguments(tcx, segment),
            });
        }
    }
    None
}

/// Whether the local impl `impl_did` bounds its context by the provider trait `provider_did` in the
/// shape [`detect_provider_trait_in_uses`] reshapes, without its context. A call of the paired
/// consumer trait's method in that impl then fails as a consequence of that one mistake.
pub fn names_provider_trait_in_uses(tcx: TyCtxt<'_>, impl_did: DefId, provider_did: DefId) -> bool {
    let Some(local) = impl_did.as_local() else {
        return false;
    };
    let ItemKind::Impl(imp) = tcx.hir_expect_item(local).kind else {
        return false;
    };
    context_trait_bounds(imp.generics).any(|path| {
        path.res.opt_def_id() == Some(provider_did)
            && path
                .segments
                .last()
                .is_some_and(|segment| lacks_only_the_context(tcx, provider_did, segment))
    })
}

/// The trait paths of the `where` bounds on an impl's generated context, from the impl's
/// `generics`.
fn context_trait_bounds<'hir>(
    generics: &'hir Generics<'hir>,
) -> impl Iterator<Item = &'hir Path<'hir>> {
    generics
        .predicates
        .iter()
        .filter_map(|predicate| match predicate.kind {
            WherePredicateKind::BoundPredicate(bound) if bounds_context(bound.bounded_ty) => {
                Some(bound.bounds)
            }
            _ => None,
        })
        .flatten()
        .filter_map(|generic_bound| match generic_bound {
            GenericBound::Trait(poly) => Some(poly.trait_ref.path),
            _ => None,
        })
}

/// Whether the provider trait `provider_did` is written without the context `#[uses]` leaves out:
/// with no type argument at all (rustc's missing generics), or with every type argument but the
/// context (`AreaCalculator<Rectangle>` for a generic component). Lifetimes are not counted, since
/// the context is a type. Any other count is the program's own arity mistake, or the full count a
/// valid bound.
fn lacks_only_the_context(tcx: TyCtxt<'_>, provider_did: DefId, segment: &PathSegment<'_>) -> bool {
    let written = segment.args.map_or(0, |args| {
        args.args
            .iter()
            .filter(|arg| matches!(arg, GenericArg::Type(_)))
            .count()
    });
    // The trait's own type parameters, `Self` excluded: the context, then the component's.
    let taken = tcx
        .generics_of(provider_did)
        .own_params
        .iter()
        .filter(|param| matches!(param.kind, GenericParamDefKind::Type { .. }))
        .count()
        .saturating_sub(1);
    written == 0 || written + 1 == taken
}

/// Whether a `where` bound's bounded type is the generated context: `Self` in a `#[cgp_fn]` impl,
/// or the `__Context__` `#[cgp_impl]` writes for it.
fn bounds_context(bounded: &rustc_hir::Ty<'_>) -> bool {
    let TyKind::Path(QPath::Resolved(None, path)) = bounded.kind else {
        return false;
    };
    matches!(path.segments, [segment]
        if segment.ident.name.as_str() == "Self"
            || segment.ident.name.as_str() == CGP_IMPL_CONTEXT_PARAM)
}
