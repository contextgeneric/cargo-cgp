//! Building the real consumer obligation an anchor seeds the walk with.

use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};
use rustc_span::def_id::DefId;

use crate::config::{CGP_FIELD_CRATE, LIFE_TYPE};
use crate::resolve::cgp_item::is_cgp_item;

/// Build `Ctx: ConsumerTrait<Params…>` from a consumer trait and the component's `Params` slot.
///
/// The slot groups a component's extra parameters as all-types data — none as the unit `()`, one
/// bare, several as a tuple, and a lifetime lifted into `Life<'a>` — but the consumer trait itself
/// wants its arguments back in their declared kinds and arity. So the slot is ungrouped against the
/// trait's *own* generics rather than by its shape alone: the parameter count decides whether a
/// tuple is *the* single (tuple-typed) parameter or several parameters to spread, and a lifetime
/// parameter takes its region back out of the `Life<'a>` lift. Building the trait ref from the
/// slot's shape alone would hand the solver a malformed obligation — spreading a single tuple-typed
/// parameter into two, or a `Life<'a>` *type* where a region belongs, the latter aborting the
/// compiler when the solver relates it. `None` when the slot cannot be matched to the trait's
/// parameters, so the caller declines to the fallback instead.
pub(crate) fn consumer_obligation<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
    consumer_did: DefId,
    params: Ty<'tcx>,
) -> Option<ty::PolyTraitPredicate<'tcx>> {
    // `own_params` opens with the implicit `Self`; the rest are the component's parameters.
    let expected = &tcx.generics_of(consumer_did).own_params[1..];

    let supplied: Vec<Ty<'tcx>> = match (expected.len(), params.kind()) {
        (0, _) if params.is_unit() => Vec::new(),
        // A single parameter is grouped bare — even when it is itself a tuple type, which is why
        // the parameter count is consulted before the slot's shape.
        (1, _) => vec![params],
        (n, ty::Tuple(elems)) if elems.len() == n => elems.iter().collect(),
        _ => return None,
    };

    let mut args: Vec<ty::GenericArg<'tcx>> = vec![context.into()];
    for (param, ty) in std::iter::zip(expected, supplied) {
        match param.kind {
            ty::GenericParamDefKind::Type { .. } => args.push(ty.into()),
            ty::GenericParamDefKind::Lifetime => args.push(life_region(tcx, ty)?.into()),
            // `#[cgp_component]` rejects const parameters, so a const here is not a CGP consumer.
            ty::GenericParamDefKind::Const { .. } => return None,
        }
    }
    let trait_ref = ty::TraitRef::new(tcx, consumer_did, args);
    Some(ty::Binder::dummy(trait_ref).upcast(tcx))
}

/// The region inside CGP's lifetime lift `Life<'a>`, or `None` when `ty` is not the genuine
/// `cgp_field::Life`.
fn life_region<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<ty::Region<'tcx>> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), LIFE_TYPE, CGP_FIELD_CRATE) {
        return None;
    }
    args.regions().next()
}
