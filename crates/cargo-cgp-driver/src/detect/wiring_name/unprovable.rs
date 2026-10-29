//! The requirements a wiring entry whose value does not resolve leaves unprovable.

use rustc_hir::{QPath, TyKind};
use rustc_middle::ty::TyCtxt;

use crate::resolve::query::marker_to_consumer;

/// The requirements a wiring entry with an unresolved value leaves unprovable, as the compiler
/// prints them up to the component's parameters: the entry's context implementing the consumer
/// trait of the component its key names (`Person: CanGreet`, what a call reports), and using the
/// component (`Person: CanUseComponent<GreeterComponent`, what a check reports, closed by `>` or
/// followed by the parameters it checks).
pub fn unprovable_requirements(
    tcx: TyCtxt<'_>,
    key: &rustc_hir::Ty<'_>,
    impl_did: rustc_hir::def_id::LocalDefId,
) -> Vec<String> {
    let TyKind::Path(QPath::Resolved(None, path)) = key.kind else {
        return Vec::new();
    };
    let Some(marker_did) = path.res.opt_def_id() else {
        return Vec::new();
    };
    let marker = tcx
        .type_of(marker_did)
        .instantiate_identity()
        .skip_norm_wip();
    let Some((consumer_did, _)) = marker_to_consumer(tcx, marker) else {
        return Vec::new();
    };
    let context = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
    vec![
        format!("{context}: {}", tcx.def_path_str(consumer_did)),
        format!(
            "{context}: CanUseComponent<{}",
            tcx.def_path_str(marker_did)
        ),
    ]
}
