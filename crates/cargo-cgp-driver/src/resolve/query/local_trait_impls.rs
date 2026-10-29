//! The trait impls the crate being compiled writes.

use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::LocalDefId;

/// Every trait impl (`impl Trait for Ty`, not an inherent impl) the local crate defines, in the
/// order the crate's items are listed. The caller keeps the impls it wants, such as those at a
/// diagnostic's span.
pub fn local_trait_impls(tcx: TyCtxt<'_>) -> impl Iterator<Item = LocalDefId> {
    tcx.hir_crate_items(())
        .definitions()
        .filter(move |&local| matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }))
}
