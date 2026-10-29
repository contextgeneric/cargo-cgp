//! A wiring entry whose generic parameters its key does not use.

use cargo_cgp_error_processing::{EntryTable, ParameterUse, StructuralMistake, WiringKey};
use rustc_data_structures::fx::FxHashSet;
use rustc_hir::def::DefKind;
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, GenericArg, GenericArgKind, GenericParamDefKind, Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{
    CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT, REDIRECT_LOOKUP_TYPE,
};
use crate::detect::conflict::{impl_delegate_type, same_range};
use crate::resolve::query::{
    describe_key, is_cgp_item, is_namespace_lookup_trait, render_lookup_trait,
};

/// The wiring entry an `E0207` at `primary_span` is about, when the unconstrained parameter there
/// is one an entry's generic list binds and its key leaves out. `<T> GreeterComponent: GreetWith<T>`
/// in the wiring of `Person` lowers to `impl<T> DelegateComponent<GreeterComponent> for Person`
/// (with an `IsProviderFor` twin, and a check impl under `delegate_and_check_components!`), and the
/// same entry in a `cgp_namespace!` to an impl of the namespace's lookup trait for its key; only
/// the key and the table can bind a parameter there, so rustc reports each one the key leaves out,
/// once per impl. The mistake names every such parameter, so each of those errors recovers the
/// same one. `None` when no entry declares the parameter at the caret.
pub fn detect_entry_parameters_not_in_key(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<StructuralMistake> {
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_did = local.to_def_id();
        let declares_param = tcx
            .generics_of(impl_did)
            .own_params
            .iter()
            .any(|param| same_range(tcx.def_span(param.def_id), primary_span));
        if !declares_param {
            continue;
        }
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        // `DelegateComponent` has a namespace trait's shape too, so it is told apart first.
        let in_context = is_cgp_item(
            tcx,
            trait_ref.def_id,
            DELEGATE_COMPONENT_TRAIT,
            CGP_COMPONENT_CRATE,
        );
        if !in_context
            && (!is_namespace_lookup_trait(tcx, trait_ref.def_id)
                || declared_by_provider_impl(tcx, impl_did, primary_span))
        {
            continue;
        }
        let unconstrained = unconstrained_params(tcx, impl_did, trait_ref);
        if !unconstrained
            .iter()
            .any(|&(_, span, _)| same_range(span, primary_span))
            || binds_loop_variable(tcx, impl_did, &unconstrained)
        {
            continue;
        }
        // A context's entry keys `DelegateComponent<Key> for Context`; a namespace's keys
        // `Namespace<…> for Key`.
        let key = if in_context {
            trait_ref.args.type_at(1)
        } else {
            trait_ref.self_ty()
        };
        let indices: Vec<u32> = unconstrained.iter().map(|&(index, _, _)| index).collect();
        let delegate = impl_delegate_type(tcx, impl_did);
        // Printed by each type's own name, as an `E0207` arrives before a trimmed print is safe
        // (see the loop detector).
        return with_forced_trimmed_paths!({
            let key = match describe_key(tcx, key, impl_did)? {
                WiringKey::Path(text) | WiringKey::Component(text) | WiringKey::Type(text) => text,
                WiringKey::Blanket(_) => return None,
            };
            let table = if in_context {
                EntryTable::Context(trait_ref.self_ty().to_string())
            } else {
                EntryTable::Namespace(render_lookup_trait(tcx, impl_did))
            };
            Some(StructuralMistake::EntryParametersNotInKey {
                table,
                key,
                parameters: unconstrained
                    .iter()
                    .map(|(_, _, name)| name.clone())
                    .collect(),
                used: parameter_use(tcx, delegate, &indices),
            })
        });
    }
    None
}

/// The impl's own type and const parameters that nothing binds, each with its index, its span, and
/// its name, in written order. A parameter is bound when the trait reference (its `Self` included)
/// names it, or when the right-hand side of an associated-type bound names it and the bound's own
/// arguments are bound. rustc applies the same rule before reporting `E0207`, except that it does
/// not count a bound on the impl's own trait; a parameter only such a bound names is treated as
/// bound here, so its error is left as rustc wrote it.
fn unconstrained_params<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_did: DefId,
    trait_ref: ty::TraitRef<'tcx>,
) -> Vec<(u32, Span, String)> {
    let mut bound: FxHashSet<u32> = trait_ref
        .args
        .iter()
        .flat_map(|arg| param_indices(arg))
        .collect();
    let projections: Vec<_> = tcx
        .clauses_of(impl_did)
        .clauses
        .iter()
        .filter_map(|(clause, _)| clause.as_projection_clause())
        .map(|projection| projection.skip_binder())
        .collect();
    loop {
        let before = bound.len();
        for projection in &projections {
            let inputs_bound = projection
                .projection_term
                .args
                .iter()
                .flat_map(|arg| param_indices(arg))
                .all(|index| bound.contains(&index));
            if inputs_bound {
                bound.extend(param_indices(projection.term.into()));
            }
        }
        if bound.len() == before {
            break;
        }
    }
    tcx.generics_of(impl_did)
        .own_params
        .iter()
        .filter(|param| {
            matches!(
                param.kind,
                GenericParamDefKind::Type { .. } | GenericParamDefKind::Const { .. }
            ) && !bound.contains(&param.index)
        })
        .map(|param| {
            (
                param.index,
                tcx.def_span(param.def_id),
                param.name.to_string(),
            )
        })
        .collect()
}

/// The indices of the type and const parameters `arg` names, at any depth.
fn param_indices(arg: GenericArg<'_>) -> Vec<u32> {
    arg.walk()
        .filter_map(|inner| match inner.kind() {
            GenericArgKind::Type(ty) => match ty.kind() {
                ty::Param(param) => Some(param.index),
                _ => None,
            },
            GenericArgKind::Const(ct) => match ct.kind() {
                ty::ConstKind::Param(param) => Some(param.index),
                _ => None,
            },
            GenericArgKind::Lifetime(_) => None,
        })
        .collect()
}

/// Whether one of the unconstrained parameters is a `for` loop's key variable: an entry a loop
/// generates is bounded by the table's lookup trait on that variable, the `CGP-E017` shape.
fn binds_loop_variable(
    tcx: TyCtxt<'_>,
    impl_did: DefId,
    unconstrained: &[(u32, Span, String)],
) -> bool {
    tcx.clauses_of(impl_did)
        .clauses
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .map(|bound| bound.skip_binder())
        .any(|bound| {
            matches!(bound.self_ty().kind(), ty::Param(param)
                if unconstrained.iter().any(|&(index, _, _)| index == param.index))
                && is_namespace_lookup_trait(tcx, bound.def_id())
        })
}

/// Whether the parameter at `primary_span` belongs to a provider impl the namespace impl was
/// copied from. A `#[default_impl]` registration copies its provider impl's generic list, so the
/// provider impl declares the same parameter; an entry the program wrote in a `cgp_namespace!` is
/// the only impl declaring its parameters.
fn declared_by_provider_impl(tcx: TyCtxt<'_>, namespace_impl: DefId, primary_span: Span) -> bool {
    tcx.hir_crate_items(()).definitions().any(|local| {
        let impl_did = local.to_def_id();
        if impl_did == namespace_impl
            || !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true })
        {
            return false;
        }
        let trait_did = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip()
            .def_id;
        let is_wiring = [DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT]
            .iter()
            .any(|name| is_cgp_item(tcx, trait_did, name, CGP_COMPONENT_CRATE))
            || is_namespace_lookup_trait(tcx, trait_did);
        !is_wiring
            && tcx
                .generics_of(impl_did)
                .own_params
                .iter()
                .any(|param| same_range(tcx.def_span(param.def_id), primary_span))
    })
}

/// What in the entry uses the parameters at `indices`: the provider when the entry maps its key
/// to a plain type that names one of them, another part of its value (the table a `->` entry
/// forwards to, a `=>` redirect's path) when only that does, and nothing otherwise.
fn parameter_use<'tcx>(
    tcx: TyCtxt<'tcx>,
    delegate: Option<Ty<'tcx>>,
    indices: &[u32],
) -> ParameterUse {
    let Some(delegate) = delegate else {
        return ParameterUse::Unused;
    };
    let uses = param_indices(delegate.into())
        .iter()
        .any(|index| indices.contains(index));
    if !uses {
        return ParameterUse::Unused;
    }
    match delegate.kind() {
        ty::Adt(def, _)
            if !is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE) =>
        {
            ParameterUse::Provider(tcx.erase_and_anonymize_regions(delegate).to_string())
        }
        _ => ParameterUse::Value,
    }
}
