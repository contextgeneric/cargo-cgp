//! The other components a wired provider is a provider of.

use cargo_cgp_error_processing::ProvidedComponent;
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _, Upcast as _};

use crate::resolve::cgp_item::provider_blanket_marker;
use crate::resolve::label::render_ty;
use crate::resolve::walk::holds;

/// How many components a `help` names at most.
const MAX_COMPONENTS: usize = 3;

/// The other components the provider of `parent` is a provider of, for the same context: a provider
/// wired under the wrong key. For each provider trait with no parameter beyond `Self` and the
/// context, other than `parent`'s own, the component whose marker its delegation blanket keys on is
/// named when the solver confirms `Provider: Trait<Context>`.
///
/// Empty when `parent` is not fully known, as for [`wrapping_adapters`].
pub(crate) fn provided_components<'tcx>(
    tcx: TyCtxt<'tcx>,
    parent: ty::TraitRef<'tcx>,
) -> Vec<ProvidedComponent> {
    let parent = tcx.erase_and_anonymize_regions(parent);
    if parent.has_non_region_infer()
        || parent.has_placeholders()
        || parent.has_param()
        || parent.has_aliases()
        || parent.references_error()
    {
        return Vec::new();
    }
    let (provider, context) = (parent.self_ty(), parent.args.type_at(1));
    let mut provided: Vec<ProvidedComponent> = Vec::new();
    for trait_did in tcx.all_traits_including_private() {
        if provided.len() == MAX_COMPONENTS {
            break;
        }
        if trait_did == parent.def_id || tcx.generics_of(trait_did).own_params.len() != 2 {
            continue;
        }
        let Some(marker) = provider_blanket_marker(tcx, trait_did) else {
            continue;
        };
        let applied = ty::TraitRef::new(tcx, trait_did, [provider, context]);
        if !holds(tcx, ty::Binder::dummy(applied).upcast(tcx)) {
            continue;
        }
        provided.push(ProvidedComponent {
            provider_trait: tcx.item_name(trait_did).to_string(),
            component: render_ty(tcx, marker),
        });
    }
    provided
}
