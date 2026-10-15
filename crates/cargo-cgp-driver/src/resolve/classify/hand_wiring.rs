//! A component wired by a hand-written `DelegateComponent` impl, with no `IsProviderFor` beside it.

use cargo_cgp_error_processing::{EntryFix, Leaf};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT};
use crate::resolve::cgp_item::{find_cgp_trait, is_cgp_item};
use crate::resolve::label::render_ty;

/// The [`Leaf::WiringWithoutProviderFor`] behind a step whose only failing requirements are
/// `IsProviderFor` bounds (`failing`), the shape of the delegation blanket a consumer call routes
/// through: its `Context: IsProviderFor<Component, Context, …>` bound fails while the delegate it
/// names provides the component. `delegate_components!` implements that bound beside each entry,
/// so a context with no `IsProviderFor` impl for the component wired it by a hand-written
/// `DelegateComponent` impl.
///
/// `None` when no failing bound is on `context`, or when the context has an `IsProviderFor` impl
/// for the component (or a blanket one over every key, as a namespace join writes), so the
/// failure lies with the delegate's own `IsProviderFor` instead.
pub(crate) fn wiring_without_provider_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    failing: &[ty::PolyTraitClause<'tcx>],
    context: Ty<'tcx>,
) -> Option<Leaf> {
    let trait_ref = failing.iter().find_map(|pred| {
        let trait_ref = pred.skip_binder().trait_ref;
        (is_cgp_item(
            tcx,
            trait_ref.def_id,
            IS_PROVIDER_FOR_TRAIT,
            CGP_COMPONENT_CRATE,
        ) && tcx.erase_and_anonymize_regions(trait_ref.self_ty()) == context)
            .then_some(trait_ref)
    })?;
    let component = trait_ref.args.type_at(1);
    if keyed_impl(tcx, trait_ref.def_id, context, component).is_some() {
        return None;
    }
    let delegate = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        .and_then(|delegate_did| keyed_impl(tcx, delegate_did, context, component))
        .and_then(|impl_did| written_delegate(tcx, impl_did));
    Some(Leaf::WiringWithoutProviderFor {
        component: render_ty(tcx, component),
        owner: render_ty(tcx, context),
        delegate: EntryFix(delegate),
    })
}

/// An impl of `trait_did` (`DelegateComponent` or `IsProviderFor`, both keyed by their first trait
/// argument) for `context`'s type whose key is `component`'s type or a parameter standing for
/// every key.
fn keyed_impl<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_did: DefId,
    context: Ty<'tcx>,
    component: Ty<'tcx>,
) -> Option<DefId> {
    let (ty::Adt(context_def, _), ty::Adt(component_def, _)) = (context.kind(), component.kind())
    else {
        return None;
    };
    tcx.all_impls(trait_did).find(|&impl_did| {
        let impl_ref = tcx.impl_trait_ref(impl_did).skip_binder();
        let for_context =
            matches!(impl_ref.self_ty().kind(), ty::Adt(def, _) if def.did() == context_def.did());
        let for_component = match impl_ref.args.type_at(1).kind() {
            ty::Adt(def, _) => def.did() == component_def.did(),
            ty::Param(_) => true,
            _ => false,
        };
        for_context && for_component
    })
}

/// The `Delegate` a hand-written `DelegateComponent` impl names, when the impl is in this crate and
/// written in the source rather than by a macro, so a `help` can tell the reader to write the same
/// entry in `delegate_components!`.
fn written_delegate(tcx: TyCtxt<'_>, impl_did: DefId) -> Option<String> {
    if !impl_did.is_local() || tcx.def_span(impl_did).from_expansion() {
        return None;
    }
    let delegate = tcx
        .associated_items(impl_did)
        .in_definition_order()
        .find(|item| matches!(item.kind, ty::AssocKind::Type { .. }))?;
    let delegate = tcx
        .type_of(delegate.def_id)
        .instantiate_identity()
        .skip_norm_wip();
    Some(render_ty(tcx, tcx.erase_and_anonymize_regions(delegate)))
}
