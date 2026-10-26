//! Whether a provider obligation is known well enough to confirm a fix or a leaf against.

use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _};

/// `parent` with its regions erased, or `None` when it holds an inference variable, a placeholder,
/// a parameter, a projection, or an error: a fix the solver confirms against such an obligation is
/// confirmed against a guess, and a mismatch found in it may be the walk's own.
pub(crate) fn known_obligation<'tcx>(
    tcx: TyCtxt<'tcx>,
    parent: ty::TraitRef<'tcx>,
) -> Option<ty::TraitRef<'tcx>> {
    let parent = tcx.erase_and_anonymize_regions(parent);
    let unknown = parent.has_non_region_infer()
        || parent.has_placeholders()
        || parent.has_param()
        || parent.has_aliases()
        || parent.references_error();
    (!unknown).then_some(parent)
}
