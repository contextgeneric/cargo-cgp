//! The unmet `DelegateComponent` lookups on the context, and the tables its entries forward to.

use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::query::{
    holds, impl_where_obligations, is_cgp_item, may_hold_for_some_unknown, trim_unknown_path_tail,
};

/// The unmet `DelegateComponent` lookups on *another* table that the context's own entry for
/// `pred`'s key forwards to, the `where Table: DelegateComponent<Key>` bound a `Key -> Table` entry
/// generates. Empty when the context has no such entry, so its missing wiring stands.
pub fn delegated_table_lookups<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Vec<ty::PolyTraitClause<'tcx>> {
    let Some(children) = impl_where_obligations(tcx, pred) else {
        return Vec::new();
    };
    children
        .into_iter()
        .filter(|child| {
            let trait_ref = child.skip_binder().trait_ref;
            is_cgp_item(
                tcx,
                trait_ref.def_id,
                DELEGATE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            ) && tcx.erase_and_anonymize_regions(trait_ref.self_ty()) != context
                && !holds(tcx, *child)
        })
        .collect()
}

/// The unmet `DelegateComponent` obligation to report when `pred` is one **on the context** — with
/// any trailing unknown segments trimmed off a redirect-path key — or `None` when `pred` is not one.
///
/// The trim is what makes a call-site-anchored redirect reportable. A `RedirectLookup` keys on the
/// path *plus the component's parameters*, so a lookup recovered from a call whose input is inferred
/// carries that placeholder as the path's last segment: the leaf would name an entry the programmer
/// could not write, and would be dropped as unknowable before it was ever shown. Trimming leaves the
/// path they can actually wire ([`trim_unknown_path_tail`]). The trimmed leaf claims the context has
/// no entry under that path for *any* value of the unknown, so it is kept only when the lookup may
/// hold for none ([`may_hold_for_some_unknown`]); otherwise `pred` is returned untrimmed, and the
/// placeholder it carries drops it.
pub fn missing_context_wiring<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
) -> Option<ty::PolyTraitClause<'tcx>> {
    let trait_ref = pred.skip_binder().trait_ref;
    if !is_cgp_item(
        tcx,
        trait_ref.def_id,
        DELEGATE_COMPONENT_TRAIT,
        CGP_COMPONENT_CRATE,
    ) {
        return None;
    }
    if tcx.erase_and_anonymize_regions(trait_ref.self_ty()) != context {
        return None;
    }

    let key = trait_ref.args.type_at(1);
    let Some(trimmed) = trim_unknown_path_tail(tcx, key) else {
        return Some(pred);
    };
    if may_hold_for_some_unknown(tcx, pred) {
        return Some(pred);
    }
    let args: [ty::GenericArg<'tcx>; 2] = [trait_ref.self_ty().into(), trimmed.into()];
    let trait_ref = ty::TraitRef::new(tcx, trait_ref.def_id, args);
    Some(ty::Binder::dummy(trait_ref).upcast(tcx))
}
