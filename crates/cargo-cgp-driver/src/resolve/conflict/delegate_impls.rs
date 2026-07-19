//! Reading the local `DelegateComponent` impls a conflict can land on.

use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

/// The value of an impl's `Delegate` associated type — what a wiring entry (a `DelegateComponent`
/// impl, or a `cgp_namespace!` entry's namespace-trait impl) maps its key to — or `None` when the
/// impl declares no `Delegate`.
pub(crate) fn impl_delegate_type<'tcx>(tcx: TyCtxt<'tcx>, impl_did: DefId) -> Option<Ty<'tcx>> {
    tcx.associated_items(impl_did)
        .in_definition_order()
        .find(|item| item.name().as_str() == "Delegate")
        .map(|item| {
            tcx.type_of(item.def_id)
                .instantiate_identity()
                .skip_norm_wip()
        })
}

/// One local `DelegateComponent` impl, read off the compiler: where it was written, the context
/// it wires, the key it maps, and its `Delegate` (the provider or redirect it maps to).
pub(crate) struct DelegateImpl<'tcx> {
    pub(crate) def_span: Span,
    pub(crate) self_ty: Ty<'tcx>,
    pub(crate) key: Ty<'tcx>,
    pub(crate) delegate: Option<Ty<'tcx>>,
    pub(crate) impl_did: DefId,
}

/// Every local impl of the `DelegateComponent` trait, read into [`DelegateImpl`]s. Only local
/// impls can be an entry the user wrote; library blanket impls sit in another crate and never
/// match a caret in the user's source.
pub(crate) fn local_delegate_impls<'tcx>(
    tcx: TyCtxt<'tcx>,
    delegate_did: DefId,
) -> Vec<DelegateImpl<'tcx>> {
    let mut impls = Vec::new();
    for impl_did in tcx.all_impls(delegate_did) {
        if !impl_did.is_local() {
            continue;
        }
        let self_ty = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        // `DelegateComponent<Key>` — args are `[Self, Key]`.
        let key = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip()
            .args
            .type_at(1);
        let delegate = impl_delegate_type(tcx, impl_did);
        impls.push(DelegateImpl {
            def_span: tcx.def_span(impl_did),
            self_ty,
            key,
            delegate,
            impl_did,
        });
    }
    impls
}
