//! Wording two colliding entries whose keys differ: which key covers the other.

use cargo_cgp_error_processing::{WiringConflict, WiringKey};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode};
use rustc_span::DUMMY_SP;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::conflict::{ConflictSide, describe_key};

/// Whether two colliding entries map one key: their keys render alike, or one is a blanket
/// forwarding over every key (a namespace join, or a namespace's inheritance blanket). Only such a
/// pair can redirect a key twice or wire it beside its own redirect.
pub(crate) fn maps_one_key<'tcx>(
    tcx: TyCtxt<'tcx>,
    first: &ConflictSide<'tcx>,
    conflicting: &ConflictSide<'tcx>,
) -> bool {
    first.is_blanket()
        || conflicting.is_blanket()
        || describe_key(tcx, first.key, first.impl_did)
            == describe_key(tcx, conflicting.key, conflicting.impl_did)
}

/// Word two colliding entries with distinct keys as an overlap, whose "already covers" key matches
/// every key the other one names. When the later entry's key covers the earlier one's, as
/// `@C.<T> Wrap<T>` written after `@C.Wrap<u64>` does, the earlier key is the covered one.
/// Otherwise the earlier key covers what the two share: the later key whole when it covers that
/// (`@C.<T> Wrap<T>` before `@C.Wrap<u64>`), or the part they share when neither covers the other
/// (`@C.Eval` and `@C.<Code> Code.u64` share `@C.Eval.u64`). `None` when the shared key cannot be
/// rendered.
pub(crate) fn overlap_conflict<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: String,
    (first, first_key): (&ConflictSide<'tcx>, WiringKey),
    (conflicting, conflicting_key): (&ConflictSide<'tcx>, WiringKey),
) -> Option<WiringConflict> {
    let (covered, covering) = if covers(tcx, conflicting, first) {
        (first_key, conflicting_key)
    } else {
        let shared = shared_key(tcx, first, conflicting)?;
        (describe_key(tcx, shared, conflicting.impl_did)?, first_key)
    };
    Some(WiringConflict::Overlap {
        context,
        conflicting: covered,
        first: covering,
    })
}

/// Whether `general`'s key matches every key `specific`'s does: the general entry's impl
/// parameters, made fresh inference variables, unify with the specific key, whose own parameters
/// stay rigid. A path entry covers the longer paths beneath it, since its key leaves the tail open.
fn covers<'tcx>(
    tcx: TyCtxt<'tcx>,
    general: &ConflictSide<'tcx>,
    specific: &ConflictSide<'tcx>,
) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let args = infcx.fresh_args_for_item(DUMMY_SP, general.impl_did);
    let pattern = ty::EarlyBinder::bind(tcx, general.key)
        .instantiate(tcx, args)
        .skip_norm_wip();
    ocx.eq(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        pattern,
        specific.key,
    )
    .is_ok()
}

/// The most general key both entries map: their keys, each with its impl's parameters made fresh
/// inference variables, unified, then the conflicting entry's key with each parameter set to the
/// value the unification gives it. A parameter that unified only with another open parameter stays
/// a parameter, as a path's open tail does, so it renders as the entry wrote it, and a part of a
/// value the earlier key leaves open renders as `_`. `None` when the keys do not unify.
fn shared_key<'tcx>(
    tcx: TyCtxt<'tcx>,
    first: &ConflictSide<'tcx>,
    conflicting: &ConflictSide<'tcx>,
) -> Option<Ty<'tcx>> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let first_args = infcx.fresh_args_for_item(DUMMY_SP, first.impl_did);
    let conflicting_args = infcx.fresh_args_for_item(DUMMY_SP, conflicting.impl_did);
    let key_at = |side: &ConflictSide<'tcx>, args| {
        ty::EarlyBinder::bind(tcx, side.key)
            .instantiate(tcx, args)
            .skip_norm_wip()
    };
    ocx.eq(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        key_at(first, first_args),
        key_at(conflicting, conflicting_args),
    )
    .ok()?;
    let args = ty::GenericArgs::for_item(tcx, conflicting.impl_did, |param, _| {
        let value = infcx.deeply_resolve_ignoring_regions(conflicting_args[param.index as usize]);
        match value.as_type() {
            Some(ty) if ty.is_ty_var() => tcx.mk_param_from_def(param),
            _ => value,
        }
    });
    let shared = key_at(conflicting, args);
    Some(tcx.erase_and_anonymize_regions(shared))
}
