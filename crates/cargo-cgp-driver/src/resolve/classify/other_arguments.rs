//! A provider that implements its provider trait, but for other arguments than the wiring gives it.

use cargo_cgp_error_processing::Leaf;
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode};
use rustc_span::DUMMY_SP;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::cgp_item::is_provider_trait;
use crate::resolve::classify::known_obligation;
use crate::resolve::label::render_ty;

/// The [`Leaf::ProviderForOtherArguments`] behind `parent`, a provider-trait obligation the walk
/// followed into the delegation blanket because no impl of the provider itself matches it: the one
/// argument no concrete impl of the provider accepts, with the arguments those impls take there.
///
/// `None` keeps the leaf the dead-end it otherwise is. That happens when `parent` is not fully
/// known (it holds an unknown a call leaves open, or a projection the walk could not reduce, so a
/// mismatch may be the walk's own), when no concrete impl's `Self` matches the provider, and when
/// no single argument explains the mismatch.
pub(crate) fn provider_for_other_arguments<'tcx>(
    tcx: TyCtxt<'tcx>,
    parent: ty::TraitRef<'tcx>,
) -> Option<Leaf> {
    if !is_provider_trait(tcx, parent.def_id) {
        return None;
    }
    let parent = known_obligation(tcx, parent)?;
    let provider = parent.self_ty();
    let ty::Adt(provider_def, _) = provider.kind() else {
        return None;
    };
    let impls: Vec<DefId> = tcx
        .all_impls(parent.def_id)
        .filter(|&impl_did| {
            matches!(
                tcx.impl_trait_ref(impl_did).skip_binder().self_ty().kind(),
                ty::Adt(def, _) if def.did() == provider_def.did()
            )
        })
        .filter(|&impl_did| accepts(tcx, impl_did, 0, provider))
        .collect();
    if impls.is_empty() {
        return None;
    }

    // Every type argument after `Self` that no impl accepts; exactly one explains the mismatch.
    let mut rejected = parent
        .args
        .iter()
        .enumerate()
        .skip(1)
        .filter_map(|(index, arg)| Some((index, arg.as_type()?)))
        .filter(|&(index, required)| {
            !impls
                .iter()
                .any(|&impl_did| accepts(tcx, impl_did, index, required))
        });
    let (index, required) = rejected.next()?;
    if rejected.next().is_some() {
        return None;
    }

    // The provider trait's context is its first type parameter after `Self`, as the labels read it.
    let context_index = parent
        .args
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, arg)| arg.as_type().map(|_| index));
    let param = (Some(index) != context_index).then(|| {
        tcx.generics_of(parent.def_id)
            .param_at(index, tcx)
            .name
            .to_string()
    });
    // Each impl's argument with its own generic parameters left open, so they print as `_`
    // (`Result<_, _>`) rather than as the names a library's impl gives them.
    let mut provided: Vec<String> = Vec::new();
    for &impl_did in &impls {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let impl_args = infcx.fresh_args_for_item(DUMMY_SP, impl_did);
        let written = tcx
            .impl_trait_ref(impl_did)
            .instantiate(tcx, impl_args)
            .skip_norm_wip()
            .args
            .type_at(index);
        let written = render_ty(tcx, tcx.erase_and_anonymize_regions(written));
        if !provided.contains(&written) {
            provided.push(written);
        }
    }
    Some(Leaf::ProviderForOtherArguments {
        provider: render_ty(tcx, provider),
        provider_trait: tcx.item_name(parent.def_id).to_string(),
        param,
        provided,
        required: render_ty(tcx, required),
    })
}

/// Whether the impl `impl_did` takes `required` as the type argument at `index` of its trait
/// reference, its own generic parameters standing for any type. The other arguments are not
/// considered, so each position is judged on its own.
fn accepts<'tcx>(tcx: TyCtxt<'tcx>, impl_did: DefId, index: usize, required: Ty<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let impl_args = infcx.fresh_args_for_item(DUMMY_SP, impl_did);
    let impl_ref = tcx
        .impl_trait_ref(impl_did)
        .instantiate(tcx, impl_args)
        .skip_norm_wip();
    ocx.eq(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        impl_ref.args.type_at(index),
        required,
    )
    .is_ok()
}
