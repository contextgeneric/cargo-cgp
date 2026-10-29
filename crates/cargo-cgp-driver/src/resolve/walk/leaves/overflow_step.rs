//! Naming why a walk was cut: the provider step and the lookup that grows.

use cargo_cgp_error_processing::{OverflowStep, WiringOverflow};
use rustc_middle::ty::{self, TyCtxt};

use crate::resolve::label::{redirect_lookup_args, trait_generics};
use crate::resolve::query::is_provider_trait;

/// The provider step `parent` that requires `required`, when `parent` is the provider-trait
/// obligation of a provider a wiring entry chose (not the context's delegation routing, whose
/// `Self` is the context). `grandparent` is the obligation above `parent`: the entry's routing, on
/// the context for a direct entry or on a `RedirectLookup` for an `open` or namespace entry. A
/// provider a higher-order provider holds inside it, such as a dispatcher's inner `UseContext`, is
/// below another provider instead, and no entry names it. The context is the provider trait's own
/// context argument, since `required` can be on another type: a lookup that grows in the type the
/// consumer trait is required of.
pub fn provider_step<'tcx>(
    tcx: TyCtxt<'tcx>,
    grandparent: Option<ty::PolyTraitClause<'tcx>>,
    parent: ty::TraitRef<'tcx>,
    required: ty::PolyTraitClause<'tcx>,
) -> Option<OverflowStep> {
    let required = required.skip_binder().trait_ref;
    if !is_provider_trait(tcx, parent.def_id) {
        return None;
    }
    let context = parent.args.types().nth(1)?;
    if parent.self_ty() == context {
        return None;
    }
    let routing = grandparent?.skip_binder().trait_ref;
    let chosen_by_entry = routing.def_id == parent.def_id
        && (routing.self_ty() == context || redirect_lookup_args(tcx, routing.self_ty()).is_some());
    if !chosen_by_entry {
        return None;
    }
    Some(OverflowStep {
        provider: tcx
            .erase_and_anonymize_regions(parent.self_ty())
            .to_string(),
        provider_trait: format!(
            "{}{}",
            tcx.item_name(parent.def_id),
            trait_generics(tcx, parent, 2)
        ),
        context: context.to_string(),
        requires: format!(
            "{}{}",
            tcx.item_name(required.def_id),
            trait_generics(tcx, required, 1)
        ),
        required_of: required.self_ty().to_string(),
    })
}

/// The growing lookup a depth cut ends: the first obligation in `prefix` that a later one repeats
/// for the same trait with other arguments, its `Self` type included, the later one being the
/// first larger requirement, and the provider step just above it. The requirement is named by its
/// trait and arguments when it grows in its arguments (`CanShow<Vec<u32>>`), and by its `Self` type
/// when that is what grows (`Vec<u32>`).
pub fn growing_lookup<'tcx>(
    tcx: TyCtxt<'tcx>,
    prefix: &[ty::PolyTraitClause<'tcx>],
) -> WiringOverflow {
    let larger = prefix.iter().enumerate().find_map(|(at, first)| {
        prefix[at + 1..]
            .iter()
            .position(|later| later.def_id() == first.def_id() && later != first)
            .map(|offset| (at, at + 1 + offset))
    });
    let Some((first, larger)) = larger else {
        return WiringOverflow::Growing {
            larger: None,
            growing: None,
        };
    };
    let larger_ref = prefix[larger].skip_binder().trait_ref;
    let grows_in_self = larger_ref.self_ty() != prefix[first].skip_binder().self_ty();
    WiringOverflow::Growing {
        larger: Some(if grows_in_self {
            larger_ref.self_ty().to_string()
        } else {
            format!(
                "{}{}",
                tcx.item_name(larger_ref.def_id),
                trait_generics(tcx, larger_ref, 1)
            )
        }),
        growing: prefix[..larger].split_last().and_then(|(parent, above)| {
            provider_step(
                tcx,
                above.last().copied(),
                parent.skip_binder().trait_ref,
                prefix[larger],
            )
        }),
    }
}
