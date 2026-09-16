//! Resolving what a blanket namespace forwarding maps one concrete key to.
//!
//! Both faces of the override conflict need the same answer. A *context* that joins a namespace
//! and then wires a bare key it registers, and a *namespace* that inherits a parent and then binds
//! a bare key the parent registers, each collide with a blanket forwarding whose `Delegate` is a
//! projection rather than a written type. Reading what that projection resolves to for the
//! colliding key is what turns both from a bare overlap into a redirect collision that names the
//! path to write instead.

use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode, Unnormalized};
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{CGP_COMPONENT_CRATE, REDIRECT_LOOKUP_TYPE};
use crate::resolve::cgp_item::is_cgp_item;
use crate::resolve::conflict::{bounding_trait_ref, render_path};

/// The redirected path a blanket forwarding maps `concrete_key` to, if it maps it to a
/// `RedirectLookup` at all. `blanket_impl_did` is the forwarding impl and `blanket_key` its
/// generic key parameter — a context's `namespace …;` join, or a namespace's own inheritance
/// blanket. The answer is recovered by normalizing the forwarded trait's `Delegate` projection for
/// that key — `<concrete_key as ParentNamespace<Table>>::Delegate` — through the trait solver, the
/// same re-entrant normalization the typed resolver uses. `None` unless the key is fully concrete
/// and the projection resolves to a `RedirectLookup`.
pub(crate) fn namespace_redirect<'tcx>(
    tcx: TyCtxt<'tcx>,
    blanket_impl_did: DefId,
    blanket_key: Ty<'tcx>,
    concrete_key: Ty<'tcx>,
) -> Option<String> {
    // Only a fully concrete key resolves through the namespace to a single value.
    if concrete_key.has_param() || concrete_key.has_non_region_infer() {
        return None;
    }
    // The blanket's bounding trait `<blanket key>: NsTrait<Ctx>`, rebuilt with the concrete key as
    // `Self` so the projection names the mapping for *this* key.
    let ns_ref = bounding_trait_ref(tcx, blanket_impl_did, blanket_key)?;
    let delegate_did = tcx
        .associated_items(ns_ref.def_id)
        .in_definition_order()
        .find(|item| item.name().as_str() == "Delegate")?
        .def_id;
    let mut args: Vec<ty::GenericArg<'tcx>> = ns_ref.args.iter().collect();
    *args.first_mut()? = concrete_key.into();

    let projection = Ty::new_projection(tcx, ty::IsRigid::No, delegate_did, args);
    let delegate = normalize(tcx, projection)?;
    let ty::Adt(def, redirect_args) = delegate.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE) {
        return None;
    }
    render_path(tcx, redirect_args.type_at(1))
}

/// Normalize `ty` through a fresh inference context, returning the resolved type — or `None` if it
/// does not resolve to a concrete type (an ambiguous or unresolved projection leaves inference vars
/// or an alias behind). Re-entering the solver mid-emission is the same technique the typed resolver
/// relies on.
fn normalize<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let normalized = ocx.normalize(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        Unnormalized::new_wip(ty),
    );
    let normalized = infcx.deeply_resolve_ignoring_regions(normalized);
    if normalized.has_non_region_infer() {
        return None;
    }
    Some(tcx.erase_and_anonymize_regions(normalized))
}
