//! Typed root-cause resolution for CGP check-trait failures.
//!
//! This is the compiler-internals half of the diagnostic replacement. When the emitter sees a
//! trait-bound error whose caret sits on a `check_components!` entry, it asks this module to
//! recover the *real* root cause(s) — and the transitive dependency chain that leads to each —
//! by walking the wiring's trait obligations rather than by reading the rendered error text.
//!
//! The flow, all DefId-anchored to the CGP crates so a same-named type from elsewhere can
//! never drive it:
//!
//! 1. A `check_components!` entry expands to `impl __CheckCtx<Marker, Params> for Ctx {}`,
//!    whose check trait carries `CanUseComponent<Marker, Params>` as a supertrait. We find
//!    the impl whose `Self` type span equals the diagnostic's primary span — that is the
//!    entry the error is about — and instantiate the supertrait with the impl's trait ref to
//!    get the concrete obligation `Ctx: CanUseComponent<Marker, Params>`.
//! 2. From that obligation we walk *down* the dependency graph: for each failing obligation we
//!    find the impl that would satisfy it and take its `where`-clause obligations as the
//!    children, keeping only the ones that do not already hold. Every branch that bottoms out
//!    on an unmet `cgp_field::HasField` is a root cause. This descent unifies against candidate
//!    impls with `fresh_args_for_item`, rather than using `SelectionContext`, which asserts
//!    against the next-generation solver the driver runs under.
//! 3. This all runs *during* trait solving — the emitter reaches the live `TyCtxt` through
//!    `ty::tls` while a check error is being emitted — yet fresh inference contexts re-entered
//!    here solve cleanly; that re-entrancy is the load-bearing assumption behind the design.
//! 4. Each root-cause path is rendered as a `cargo tree`-style [`DependencyTree`] with every CGP
//!    wiring trait replaced by its human form (`CanUseComponent`→consumer trait, `IsProviderFor`
//!    →provider, `HasField`→missing field), and the field name is decoded structurally from its
//!    `Symbol!`. When no branch reaches a missing field the resolver yields `None`, and the
//!    caller falls back to the untouched text-rewrite pipeline.

use cargo_cgp_error_processing::ComponentTraitNames;
use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::tree::DependencyTree;
use rustc_hir::ItemKind;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span};
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE,
    DELEGATE_COMPONENT_TRAIT, HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};

/// A recovered root cause: one missing field, with the dependency chain that needs it.
pub struct MissingField {
    /// The missing field name, decoded from its `Symbol!`, e.g. `height`.
    pub field: String,
    /// The transitive dependency chain from the checked component down to this field, rendered
    /// as a single spine.
    pub tree: DependencyTree,
}

/// The recovered root cause(s) of a check failure, in owned form so they outlive the inference
/// contexts they were read from. Today the only kind is missing fields; more will join it.
pub enum RootCause {
    /// The context is missing one or more fields its wiring needs. Each entry is an independent
    /// root cause the emitter renders as its own sub-error.
    MissingFields {
        /// The context type that lacks the field(s), e.g. `Rectangle`.
        context: String,
        /// One entry per distinct missing field, in first-seen order.
        causes: Vec<MissingField>,
    },
}

/// Bound on how deep the dependency-graph walk descends before giving up, so a pathological or
/// cyclic wiring cannot make it loop. Real dependency chains are far shorter than this.
const MAX_DEPTH: u32 = 32;

/// Whether `def_id` is a trait/type named `name` defined by crate `krate` — the DefId anchor
/// that keeps a same-named item from an unrelated crate from driving resolution, exactly as
/// `component_map::is_cgp_is_provider_for` does for `IsProviderFor`.
fn is_cgp_item(tcx: TyCtxt<'_>, def_id: DefId, name: &str, krate: &str) -> bool {
    tcx.item_name(def_id).as_str() == name && tcx.crate_name(def_id.krate).as_str() == krate
}

/// Resolve the root cause(s) of the check failure whose diagnostic caret sits at `primary_span`,
/// or `None` if this is not a resolvable `CanUseComponent` check failure (in which case the
/// caller leaves the original diagnostic to the text-rewrite fallback). `names` supplies the
/// consumer/provider trait names the dependency tree renders CGP markers as.
pub fn resolve_check_failure(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    names: &ComponentNameMap,
) -> Option<RootCause> {
    for trait_did in tcx.all_traits_including_private() {
        let Some(super_clause) = can_use_component_supertrait(tcx, trait_did) else {
            continue;
        };

        for impl_did in tcx.all_impls(trait_did) {
            // The entry the error is about is the impl whose `Self` type carries the caret's
            // span (the macro re-spans the context type onto the entry, so they coincide).
            if impl_self_ty_span(tcx, impl_did) != Some(primary_span) {
                continue;
            }

            let trait_ref = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip();
            let concrete = super_clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));

            if let Some(cause) = resolve_missing_fields(tcx, concrete, names) {
                return Some(cause);
            }
        }
    }
    None
}

/// The `CanUseComponent<..>` supertrait clause of `trait_did`, if it carries one — the marker
/// of a `check_components!` check trait. Anchored by DefId to `cgp_component`.
fn can_use_component_supertrait(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<ty::Clause<'_>> {
    for &(clause, _) in tcx.explicit_super_predicates_of(trait_did).skip_binder() {
        if let Some(tp) = clause.as_trait_clause()
            && is_cgp_item(
                tcx,
                tp.def_id(),
                CAN_USE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        {
            return Some(clause);
        }
    }
    None
}

/// The source span of an impl's `Self` type, e.g. the `Rectangle` in
/// `impl __CheckRectangle<..> for Rectangle {}` — which the check macro re-spans onto the
/// `check_components!` entry, so it matches the failing diagnostic's primary span.
fn impl_self_ty_span(tcx: TyCtxt<'_>, impl_did: DefId) -> Option<Span> {
    let local = impl_did.as_local()?;
    match tcx.hir_expect_item(local).kind {
        ItemKind::Impl(imp) => Some(imp.self_ty.span),
        _ => None,
    }
}

/// Walk the dependency graph of `concrete` (`Ctx: CanUseComponent<Marker, Params>`) and, for each
/// distinct missing field it bottoms out on, return that field with its rendered dependency
/// chain. `None` when no branch reaches a missing field.
fn resolve_missing_fields<'tcx>(
    tcx: TyCtxt<'tcx>,
    concrete: ty::Clause<'tcx>,
    names: &ComponentNameMap,
) -> Option<RootCause> {
    let top = concrete.as_trait_clause()?;
    let context = tcx.erase_and_anonymize_regions(top.skip_binder().self_ty());

    let mut causes = Vec::new();
    for path in collect_field_paths(tcx, top, &[], 0) {
        let leaf = path.last()?.skip_binder().trait_ref;
        let Some(field) = decode_symbol(tcx, leaf.args.type_at(1)) else {
            continue;
        };
        // One sub-error per distinct field: a field wanted by several branches is one fix.
        if causes.iter().any(|c: &MissingField| c.field == field) {
            continue;
        }
        let labels: Vec<String> = path
            .iter()
            .filter_map(|pred| label_for(tcx, *pred, context, names))
            .collect();
        if let Some(tree) = spine(labels) {
            causes.push(MissingField { field, tree });
        }
    }

    if causes.is_empty() {
        return None;
    }
    Some(RootCause::MissingFields {
        context: context.to_string(),
        causes,
    })
}

/// Collect every root→leaf path that bottoms out on an unmet `HasField`, by descending the
/// failing obligation's dependency graph. `pred` is a failing obligation and `prefix` is the path
/// of predicates above it; a `HasField` completes a path, and any other obligation contributes
/// the `where`-clause obligations of the impl that would satisfy it, recursing into just the ones
/// that do not already hold. Following *every* unmet dependency (not one) is what surfaces
/// independent missing fields as separate paths. Bounded by [`MAX_DEPTH`].
fn collect_field_paths<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    prefix: &[ty::PolyTraitPredicate<'tcx>],
    depth: u32,
) -> Vec<Vec<ty::PolyTraitPredicate<'tcx>>> {
    if depth > MAX_DEPTH {
        return Vec::new();
    }

    let mut path = prefix.to_vec();
    path.push(pred);

    if is_has_field(tcx, pred) {
        return vec![path];
    }

    let mut paths = Vec::new();
    for nested in impl_where_obligations(tcx, pred) {
        if !holds(tcx, nested) {
            paths.extend(collect_field_paths(tcx, nested, &path, depth + 1));
        }
    }
    paths
}

/// The instantiated `where`-clause trait obligations of the impl that would satisfy `obligation`
/// — its direct dependencies. Found by unifying `obligation` with each candidate impl's trait ref
/// (the next-solver-safe `fresh_args_for_item` + `eq` dance `SelectionContext` is unavailable
/// for), then instantiating and normalizing that impl's predicates. Empty when no impl matches.
fn impl_where_obligations<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
) -> Vec<ty::PolyTraitPredicate<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let obligation_ref = obligation.skip_binder().trait_ref;

    for impl_did in tcx.all_impls(obligation.def_id()) {
        let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
        let ocx = ObligationCtxt::new(&infcx);

        let impl_args = infcx.fresh_args_for_item(DUMMY_SP, impl_did);
        let impl_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate(tcx, impl_args)
            .skip_norm_wip();
        let impl_ref = ocx.normalize(
            &ObligationCause::dummy(),
            param_env,
            Unnormalized::new_wip(impl_ref),
        );
        if ocx
            .eq(
                &ObligationCause::dummy(),
                param_env,
                obligation_ref,
                impl_ref,
            )
            .is_err()
        {
            continue;
        }

        let mut obligations = Vec::new();
        for (predicate, _) in tcx.predicates_of(impl_did).instantiate(tcx, impl_args) {
            let clause: ty::Clause<'tcx> =
                ocx.normalize(&ObligationCause::dummy(), param_env, predicate);
            let clause = infcx.resolve_vars_if_possible(clause);
            // A predicate that still carries inference vars (an unconstrained impl parameter)
            // cannot be re-evaluated in a fresh context; region vars are simply erased.
            if clause.has_non_region_infer() {
                continue;
            }
            if let Some(tp) = tcx.erase_and_anonymize_regions(clause).as_trait_clause() {
                obligations.push(tp);
            }
        }
        return obligations;
    }
    Vec::new()
}

/// Whether `pred` already holds — a dependency that is satisfied and so is not descended into.
fn holds<'tcx>(tcx: TyCtxt<'tcx>, pred: ty::PolyTraitPredicate<'tcx>) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), ty::ParamEnv::empty(), pred);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// Whether a trait predicate is a genuine CGP `HasField` bound — the missing-field leaf.
fn is_has_field(tcx: TyCtxt<'_>, pred: ty::PolyTraitPredicate<'_>) -> bool {
    is_cgp_item(
        tcx,
        pred.skip_binder().def_id(),
        HAS_FIELD_TRAIT,
        CGP_FIELD_CRATE,
    )
}

/// Fold a path's rendered labels into a single-spine dependency tree, root first.
fn spine(labels: Vec<String>) -> Option<DependencyTree> {
    let mut rev = labels.into_iter().rev();
    let mut node = DependencyTree::leaf(rev.next()?);
    for label in rev {
        node = DependencyTree::node(label, vec![node]);
    }
    Some(node)
}

/// The human-readable label for one predicate in a dependency path, replacing each CGP wiring
/// trait with the concept it stands for: `CanUseComponent` with the consumer trait, `IsProviderFor`
/// with the concrete provider (and its provider trait), `HasField` with the missing field. Any
/// other trait — a user's own consumer or getter capability — is shown by name.
///
/// The steps that carry no information for a reader are dropped so the chain stays legible: the
/// `DelegateComponent` wiring table, an `IsProviderFor` for the *context itself* (the delegation
/// routing, as opposed to the real provider), and a provider trait applied directly (which every
/// `IsProviderFor` node already stands for).
fn label_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
    names: &ComponentNameMap,
) -> Option<String> {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;

    if is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE) {
        let consumer = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.consumer);
        Some(format!(
            "`{}` uses consumer trait `{consumer}`",
            trait_ref.self_ty()
        ))
    } else if is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE) {
        // Drop the routing `IsProviderFor` for the context itself; keep the real providers.
        if trait_ref.self_ty() == context {
            return None;
        }
        let provider = trait_ref.self_ty().to_string();
        let provider_trait = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.provider);
        Some(format!(
            "provider `{provider}` (provider trait `{provider_trait}`)"
        ))
    } else if is_cgp_item(tcx, did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
        let field = decode_symbol(tcx, trait_ref.args.type_at(1))?;
        Some(format!("missing field `{field}`"))
    } else if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
    {
        None
    } else {
        Some(format!("requires `{}`", tcx.item_name(did)))
    }
}

/// Whether `def_id` is a CGP *provider* trait — one carrying an `IsProviderFor` supertrait. A
/// bare provider-trait obligation (`Ctx: SomeProvider<Ctx>`) is redundant with the
/// `IsProviderFor` node that stands for the same step, so the tree drops it.
fn is_provider_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.explicit_super_predicates_of(def_id)
        .skip_binder()
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|tp| is_cgp_item(tcx, tp.def_id(), IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE))
}

/// Resolve a component marker type to its consumer or provider trait name through the name map,
/// falling back to the marker's own name when the component is not fully recognized.
fn marker_role(
    tcx: TyCtxt<'_>,
    marker: Ty<'_>,
    names: &ComponentNameMap,
    role: impl Fn(ComponentTraitNames) -> String,
) -> String {
    match adt_name(tcx, marker) {
        Some(name) => names.get(&name).map(role).unwrap_or(name),
        None => marker.to_string(),
    }
}

/// The item name of `ty` when it is an ADT (`Rectangle`, `AreaCalculatorComponent`, …).
fn adt_name(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    match ty.kind() {
        ty::Adt(def, _) => Some(tcx.item_name(def.did()).to_string()),
        _ => None,
    }
}

/// Decode a CGP `Symbol!` type into its string, by walking the `Chars<'c', Tail>` spine and
/// reading each `char` const argument until `Nil`. Anchored to `cgp_base_types`, and returns
/// `None` for any type that is not a well-formed `Symbol`.
fn decode_symbol(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), "Symbol", CGP_BASE_TYPES_CRATE) {
        return None;
    }

    // `Symbol<const LEN, Chars>` — the second argument is the head of the `Chars` spine.
    let mut current = args.type_at(1);
    let mut name = String::new();
    loop {
        let ty::Adt(def, args) = current.kind() else {
            return None;
        };
        if is_cgp_item(tcx, def.did(), "Nil", CGP_BASE_TYPES_CRATE) {
            break;
        }
        if !is_cgp_item(tcx, def.did(), "Chars", CGP_BASE_TYPES_CRATE) {
            return None;
        }

        // `Chars<const CHAR: char, Tail>` — read the char, then follow the tail.
        let scalar = args.const_at(0).try_to_value()?.valtree.try_to_leaf()?;
        name.push(char::from_u32(scalar.to_u32())?);
        current = args.type_at(1);
    }
    Some(name)
}
