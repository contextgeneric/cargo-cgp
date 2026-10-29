//! The traits that declare an associated function of a given name.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;
use rustc_span::def_id::DefId;

/// Every trait, in every crate and in the compiler's order, that declares an associated item named
/// `name` which `accepts` (a function, with or without a `self` receiver as the caller needs), each
/// paired with the first such item's `DefId`. The caller keeps the kinds of trait it wants and the
/// order it tries them in.
pub fn trait_fns_named<'tcx>(
    tcx: TyCtxt<'tcx>,
    name: Symbol,
    accepts: impl Fn(&ty::AssocItem) -> bool,
) -> impl Iterator<Item = (DefId, DefId)> {
    tcx.all_traits_including_private()
        .filter_map(move |trait_did| {
            let fn_did = tcx
                .associated_items(trait_did)
                .filter_by_name_unhygienic(name)
                .find(|item| accepts(item))?
                .def_id;
            Some((trait_did, fn_did))
        })
}
