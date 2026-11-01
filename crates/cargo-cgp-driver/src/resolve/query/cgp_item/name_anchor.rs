//! Finding a CGP item by its name and the crate that defines it.

use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;

/// Whether `def_id` is a trait/type named `name` defined by crate `krate`: the DefId anchor
/// that keeps a same-named item from an unrelated crate from driving resolution, exactly as
/// `component_map::is_cgp_is_provider_for` does for `IsProviderFor`.
pub fn is_cgp_item(tcx: TyCtxt<'_>, def_id: DefId, name: &str, krate: &str) -> bool {
    tcx.item_name(def_id).as_str() == name && tcx.crate_name(def_id.krate).as_str() == krate
}

/// The `DefId` of the CGP trait named `name` defined by crate `krate`, or `None` if the crate does
/// not use CGP. Anchored by name *and* crate, like every other CGP lookup here.
pub fn find_cgp_trait(tcx: TyCtxt<'_>, name: &str, krate: &str) -> Option<DefId> {
    tcx.all_traits_including_private()
        .find(|&did| is_cgp_item(tcx, did, name, krate))
}
