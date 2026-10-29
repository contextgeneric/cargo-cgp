//! A check entry that names no parameters for a component that takes some.
//!
//! `check_components!` and `delegate_and_check_components!` check a component at the parameters an
//! entry names, and an entry that names none is checked at the unit parameters `()`. For a component
//! with type parameters (`CanCalculateArea<Shape>`), or a key whose marker carries one
//! (`<I> FooKeyComponent<I>`), that asks whether the context can use the component at `()`, which a
//! provider written for particular parameters does not serve, so the check fails with an `E0277` on
//! `Provider: IsProviderFor<Marker, Context>` that reads like a broken provider.

use cargo_cgp_error_processing::{CheckForm, StructuralMistake, invokes_macro};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode};
use rustc_span::def_id::DefId;
use rustc_span::hygiene::{ExpnKind, MacroKind};
use rustc_span::{DUMMY_SP, Span};
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{
    CGP_COMPONENT_CRATE, DELEGATE_AND_CHECK_COMPONENTS_MACRO, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::query::{
    can_use_component_supertrait, find_cgp_trait, impl_self_ty_span, marker_to_consumer,
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
    for trait_did in tcx.all_traits_including_private() {
        let Some(super_clause) = can_use_component_supertrait(tcx, trait_did) else {
            continue;
        };
        for impl_did in tcx.all_impls(trait_did) {
            if impl_self_ty_span(tcx, impl_did) != Some(primary_span) {
                continue;
            }
            let trait_ref = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip();
            let can_use = super_clause
                .instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref))
                .as_trait_clause()?
                .skip_binder()
                .trait_ref;
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
            let parameters = checkable_parameters(tcx, impl_did, can_use.def_id, context, marker);
            if parameters.is_empty() {
                return None;
            }
            return Some(StructuralMistake::CheckAtUnitParameters {
                context: with_forced_trimmed_paths!(context.to_string()),
                entry: tcx.sess.source_map().span_to_snippet(primary_span).ok()?,
                parameters,
                form: check_form(tcx, trait_did),
            });
        }
    }
    None
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
        let impl_args = infcx.fresh_args_for_item(DUMMY_SP, impl_did);
        let impl_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate(tcx, impl_args)
            .skip_norm_wip();
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
/// `check_components!` after the component.
fn check_form(tcx: TyCtxt<'_>, trait_did: DefId) -> CheckForm {
    let from_delegate_and_check = tcx.def_span(trait_did).macro_backtrace().any(|expansion| {
        matches!(expansion.kind, ExpnKind::Macro(MacroKind::Bang, name)
            if invokes_macro(name.as_str(), DELEGATE_AND_CHECK_COMPONENTS_MACRO))
    });
    if from_delegate_and_check {
        CheckForm::DelegateAndCheckComponents
    } else {
        CheckForm::CheckComponents
    }
}
