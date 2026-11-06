//! Reading the requirements of a provider impl whose `#[cgp_impl]` header names the consumer trait.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::CGP_IMPL_CONTEXT_PARAM;
use crate::resolve::query::{
    consumer_provider_trait, is_provider_trait, local_trait_impls, owner_has_impl_of,
    written_order_clauses,
};

/// The requirements the provider of `provider_ref` states in an impl it wrote with the component's
/// consumer trait in its `#[cgp_impl]` header, the mistake `[CGP-E032]` reports: that impl's
/// clauses, in written order and region-erased, with its context parameter set to the obligation's
/// context.
///
/// Such an impl gives the provider no provider-trait impl, so the obligation fails as if the
/// provider were not one, which is a consequence of the header. But the impl still states what the
/// provider needs from the context, and a need that fails is a mistake of its own, such as a field
/// the context lacks, that would remain once the header is fixed. `None` when `provider_ref` is not
/// a provider-trait obligation on a local provider struct, when the provider implements the
/// provider trait itself, or when it has no such impl, or several.
pub fn misdirected_impl_clauses<'tcx>(
    tcx: TyCtxt<'tcx>,
    provider_ref: ty::TraitRef<'tcx>,
) -> Option<Vec<ty::Clause<'tcx>>> {
    // The structural checks run before the trait lookups, since the walk asks this of every node.
    let provider = tcx.erase_and_anonymize_regions(provider_ref.self_ty());
    let ty::Adt(provider_def, _) = provider.kind() else {
        return None;
    };
    if !provider_def.did().is_local() || !is_provider_trait(tcx, provider_ref.def_id) {
        return None;
    }
    // A provider trait's arguments are its lifetimes, the context, then the component's own types,
    // so the context is the first type argument after `Self`.
    let context = provider_ref.args.types().nth(1)?;
    if tcx.erase_and_anonymize_regions(context) == provider {
        return None;
    }
    if owner_has_impl_of(tcx, provider_ref.def_id, provider) {
        return None;
    }

    let mut found = local_trait_impls(tcx)
        .map(|local| local.to_def_id())
        .filter(|&impl_did| {
            is_misdirected_impl(tcx, impl_did, provider_ref.def_id, provider_def.did())
        });
    let impl_did = found.next()?;
    if found.next().is_some() {
        return None;
    }

    // The impl's only type parameter is its context ([`is_misdirected_impl`]), and its lifetimes
    // are erased like every obligation the walk reads.
    let args = ty::GenericArgs::for_item(tcx, impl_did, |param, _| match param.kind {
        ty::GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
        _ => context.into(),
    });
    if tcx.erase_and_anonymize_regions(tcx.type_of(impl_did).instantiate(tcx, args).skip_norm_wip())
        != provider
    {
        return None;
    }
    Some(
        written_order_clauses(tcx, impl_did, args)
            .into_iter()
            .map(|clause| tcx.erase_and_anonymize_regions(clause.skip_norm_wip()))
            .collect(),
    )
}

/// Whether the local trait impl `impl_did` is a `#[cgp_impl]` impl on the provider `provider_did`
/// whose header names the consumer trait paired with `provider_trait`: its `Self` is the provider,
/// its trait is that consumer trait, and its type parameters are the context parameter
/// `#[cgp_impl]` inserts alone.
fn is_misdirected_impl(
    tcx: TyCtxt<'_>,
    impl_did: DefId,
    provider_trait: DefId,
    provider_did: DefId,
) -> bool {
    let generics = tcx.generics_of(impl_did);
    let mut has_context = false;
    for param in &generics.own_params {
        match param.kind {
            ty::GenericParamDefKind::Lifetime => {}
            ty::GenericParamDefKind::Type { .. }
                if param.name.as_str() == CGP_IMPL_CONTEXT_PARAM =>
            {
                has_context = true
            }
            _ => return false,
        }
    }
    if !has_context || generics.parent.is_some() {
        return false;
    }
    if !matches!(
        tcx.type_of(impl_did).skip_binder().kind(),
        ty::Adt(def, _) if def.did() == provider_did
    ) {
        return false;
    }
    let header = tcx.impl_trait_ref(impl_did).skip_binder().def_id;
    consumer_provider_trait(tcx, header) == Some(provider_trait)
}
