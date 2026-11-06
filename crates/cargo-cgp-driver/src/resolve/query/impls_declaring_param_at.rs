//! The local trait impls that declare a generic parameter at a span.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use super::{local_trait_impls, same_range};

/// Each local trait impl that declares one of its own generic parameters at `span`, such as the
/// parameter an `E0207` caret points at, with that parameter and the impl's trait reference, in the
/// order of [`local_trait_impls`](fn@local_trait_impls). A macro that copies a generic list writes
/// the same parameter span into several impls, so a caller filters for the one it is about.
pub fn impls_declaring_param_at<'tcx>(
    tcx: TyCtxt<'tcx>,
    span: Span,
) -> impl Iterator<Item = (DefId, &'tcx ty::GenericParamDef, ty::TraitRef<'tcx>)> {
    local_trait_impls(tcx).filter_map(move |local| {
        let impl_did = local.to_def_id();
        let param = tcx
            .generics_of(impl_did)
            .own_params
            .iter()
            .find(|param| same_range(tcx.def_span(param.def_id), span))?;
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        Some((impl_did, param, trait_ref))
    })
}
