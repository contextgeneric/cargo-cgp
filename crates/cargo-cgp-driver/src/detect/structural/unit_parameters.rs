//! A check entry that names no parameters for a component that takes some.
//!
//! `check_components!` and `delegate_and_check_components!` check a component at the parameters an
//! entry names, and an entry that names none is checked at the unit parameters `()`. For a component
//! with type parameters (`CanCalculateArea<Shape>`), or a key whose marker carries one
//! (`<I> FooKeyComponent<I>`), that asks whether the context can use the component at `()`, which a
//! provider written for particular parameters does not serve, so the check fails with an `E0277` on
//! `Provider: IsProviderFor<Marker, Context>` that reads like a broken provider.

use cargo_cgp_error_processing::{CheckForm, StructuralMistake};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode};
use rustc_span::Span;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use super::check_macro_at;
use crate::config::{CGP_COMPONENT_CRATE, IS_PROVIDER_FOR_TRAIT};
use crate::resolve::query::{
    can_use_component_supertrait, check_impls_at, find_cgp_trait, fresh_impl_ref,
    marker_to_consumer,
};

/// Recognize an `E0277` at `primary_span` as a check entry that names no parameters for a component
/// that takes some: the caret is the entry of a check impl (whose trait has `CanUseComponent` as a
/// supertrait), its parameters are `()`, the marker's consumer trait takes parameters, and the
/// context can be checked at some parameters a provider of the component implements. Those
/// parameters, checked to hold in the check impl's environment, are the fix. `None` otherwise.
pub fn detect_check_at_unit_parameters(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<StructuralMistake> {
    let check = check_impls_at(tcx, primary_span, can_use_component_supertrait).next()?;
    let can_use = check.asserted.as_trait_clause()?.skip_binder().trait_ref;
    // `CanUseComponent<Marker, Params>`, with args `[Context, Marker, Params]`.
    let context = can_use.self_ty();
    let marker = can_use.args.type_at(1);
    if !can_use.args.type_at(2).is_unit() {
        return None;
    }
    let (consumer_did, _) = marker_to_consumer(tcx, marker)?;
    if tcx.generics_of(consumer_did).own_params.len() <= 1 {
        return None;
    }
    let parameters = checkable_parameters(tcx, check.impl_did, can_use.def_id, context, marker);
    if parameters.is_empty() {
        return None;
    }
    Some(StructuralMistake::CheckAtUnitParameters {
        context: with_forced_trimmed_paths!(context.to_string()),
        entry: tcx.sess.source_map().span_to_snippet(primary_span).ok()?,
        parameters,
        form: check_form(tcx, check.trait_did),
    })
}

/// The parameters the context can be checked at for `marker`: the `Params` of each `IsProviderFor`
/// impl whose component and context unify with the check's, fully resolved (a provider generic over
/// every parameter offers none), and kept only when `Context: CanUseComponent<Marker, Params>` holds
/// in the check impl's environment. Each is written as the compiler prints it, deduplicated, in impl
/// order.
fn checkable_parameters<'tcx>(
    tcx: TyCtxt<'tcx>,
    check_impl: DefId,
    can_use_did: DefId,
    context: Ty<'tcx>,
    marker: Ty<'tcx>,
) -> Vec<String> {
    let Some(is_provider_for) = find_cgp_trait(tcx, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
    else {
        return Vec::new();
    };
    let param_env = tcx.param_env(check_impl);
    let cause = ObligationCause::dummy();
    let mut parameters = Vec::new();
    for impl_did in tcx.all_impls(is_provider_for) {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let ocx = ObligationCtxt::new(&infcx);
        let (_, impl_ref) = fresh_impl_ref(&infcx, impl_did);
        // `IsProviderFor<Component, Context, Params>`, with args `[Provider, Component, Context,
        // Params]`.
        if ocx
            .eq(&cause, param_env, impl_ref.args.type_at(1), marker)
            .is_err()
            || ocx
                .eq(&cause, param_env, impl_ref.args.type_at(2), context)
                .is_err()
        {
            continue;
        }
        let params = infcx.deeply_resolve_ignoring_regions(impl_ref.args.type_at(3));
        if params.has_infer() || params.is_unit() {
            continue;
        }
        let check = ty::TraitRef::new(tcx, can_use_did, [context, marker, params]);
        let obligation = Obligation::new(tcx, cause.clone(), param_env, check);
        let check_infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        if !check_infcx.predicate_must_hold_modulo_regions(&obligation) {
            continue;
        }
        let written = with_forced_trimmed_paths!(params.to_string());
        if !parameters.contains(&written) {
            parameters.push(written);
        }
    }
    parameters
}

/// The macro the check trait `trait_did` comes from, read from its definition's expansion:
/// `delegate_and_check_components!` writes parameters in `#[check_params(…)]`, and
/// `check_components!` after the component, which a definition neither macro wrote is read as.
fn check_form(tcx: TyCtxt<'_>, trait_did: DefId) -> CheckForm {
    check_macro_at(tcx.def_span(trait_did)).unwrap_or(CheckForm::CheckComponents)
}
