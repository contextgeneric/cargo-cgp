//! Typed root-cause resolution for CGP check-trait failures.
//!
//! This is the compiler-internals half of the diagnostic replacement. When the emitter sees a
//! trait-bound error whose caret sits on a `check_components!` entry, it asks this module to
//! recover the *real* root cause — and the whole transitive dependency chain that leads to it —
//! by re-running the check obligation through the trait solver rather than by reading the
//! rendered error text.
//!
//! The flow, all DefId-anchored to the CGP crates so a same-named type from elsewhere can
//! never drive it:
//!
//! 1. A `check_components!` entry expands to `impl __CheckCtx<Marker, Params> for Ctx {}`,
//!    whose check trait carries `CanUseComponent<Marker, Params>` as a supertrait. We find
//!    the impl whose `Self` type span equals the diagnostic's primary span — that is the
//!    entry the error is about — and instantiate the supertrait with the impl's trait ref to
//!    get the concrete obligation `Ctx: CanUseComponent<Marker, Params>`.
//! 2. We solve that obligation in a fresh `ObligationCtxt`. This runs *during* trait solving —
//!    the emitter reaches the live `TyCtxt` through `ty::tls` while a check error is being
//!    emitted — yet a fresh inference context re-entered here solves cleanly; that re-entrancy
//!    is the load-bearing assumption behind the whole design.
//! 3. We walk the failing obligation's cause chain to recover every dependency step from the
//!    check down toward the leaf, re-solving through intermediate CGP wiring obligations
//!    (`IsProviderFor`/`CanUseComponent`) when the solver reports the failure one layer short
//!    of the `HasField` leaf. The result is an ordered chain of trait predicates; when it
//!    bottoms out on a genuine `cgp_field::HasField`, that leaf is the root cause. Its `Symbol!`
//!    argument is decoded structurally (walking the `Chars` spine) into the field name, and the
//!    whole chain is rendered as a `cargo tree`-style [`DependencyTree`] with each CGP wiring
//!    trait replaced by its human form. Anything else yields `None`, and the caller falls back
//!    to the untouched text-rewrite pipeline.

use cargo_cgp_error_processing::ComponentTraitNames;
use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::tree::DependencyTree;
use rustc_hir::ItemKind;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::traits::ObligationCauseCode;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode, Upcast};
use rustc_span::Span;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{Obligation, ObligationCause, ObligationCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE,
    DELEGATE_COMPONENT_TRAIT, HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};

/// A recovered root cause, in owned form so it outlives the inference context it was read
/// from. Today the only variant is a missing `HasField`; more leaf kinds will join it.
pub enum RootCause {
    /// A provider the context is wired to needs a field the context's struct does not have.
    MissingField {
        /// The context type that lacks the field, e.g. `Rectangle`.
        context: String,
        /// The missing field name, decoded from the `Symbol!`, e.g. `height`.
        field: String,
        /// The transitive dependency chain from the checked component down to the missing
        /// field, ready to render as one dependency note.
        tree: DependencyTree,
    },
}

/// Whether `def_id` is a trait/type named `name` defined by crate `krate` — the DefId anchor
/// that keeps a same-named item from an unrelated crate from driving resolution, exactly as
/// `component_map::is_cgp_is_provider_for` does for `IsProviderFor`.
fn is_cgp_item(tcx: TyCtxt<'_>, def_id: DefId, name: &str, krate: &str) -> bool {
    tcx.item_name(def_id).as_str() == name && tcx.crate_name(def_id.krate).as_str() == krate
}

/// Resolve the root cause of the check failure whose diagnostic caret sits at `primary_span`,
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

            if let Some(cause) = resolve_missing_field(tcx, concrete, names) {
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

/// Bound on how many CGP obligations the resolver descends through before giving up, so a
/// pathological or cyclic wiring cannot make the descent loop. Real dependency chains are far
/// shorter than this.
const MAX_DEPTH: u32 = 32;

/// Solve the concrete `Ctx: CanUseComponent<Marker, Params>` obligation, and if its dependency
/// chain bottoms out on a `HasField` leaf, return that missing field together with the rendered
/// chain that leads to it.
fn resolve_missing_field<'tcx>(
    tcx: TyCtxt<'tcx>,
    concrete: ty::Clause<'tcx>,
    names: &ComponentNameMap,
) -> Option<RootCause> {
    let chain = build_chain(tcx, concrete.as_predicate())?;

    // The chain is only a root cause we replace when it ends at a genuine `HasField`.
    let leaf = chain.last()?.skip_binder().trait_ref;
    if !is_cgp_item(tcx, leaf.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
        return None;
    }
    let context_ty = leaf.self_ty();
    let field = decode_symbol(tcx, leaf.args.type_at(1))?;
    let tree = build_tree(tcx, &chain, context_ty, names)?;

    Some(RootCause::MissingField {
        context: context_ty.to_string(),
        field,
        tree,
    })
}

/// Recover the ordered dependency chain — root check first, missing leaf last — behind a failing
/// obligation. Each solve reports the failure at a leaf and carries a cause chain of its
/// ancestors; when that leaf is an intermediate CGP wiring obligation rather than the final
/// `HasField`, we re-solve it to descend one layer deeper, stitching each segment onto the chain
/// until a `HasField` surfaces or [`MAX_DEPTH`] is reached. Descent is confined to CGP wiring
/// traits, so an unmet *ordinary* bound simply ends the chain (and the caller declines to
/// replace the diagnostic).
fn build_chain<'tcx>(
    tcx: TyCtxt<'tcx>,
    top: ty::Predicate<'tcx>,
) -> Option<Vec<ty::PolyTraitPredicate<'tcx>>> {
    let mut chain: Vec<ty::PolyTraitPredicate<'tcx>> = Vec::new();
    let mut current = top;

    for _ in 0..=MAX_DEPTH {
        let (leaf, parents) = solve_leaf(tcx, current)?;

        // `parents` runs leaf→root; reversed and with the leaf appended it is this segment
        // root→leaf. Its first node repeats the previous segment's leaf, so drop that overlap.
        let mut segment: Vec<_> = parents.into_iter().rev().collect();
        segment.push(leaf);
        if chain.last() == segment.first() {
            segment.remove(0);
        }
        chain.extend(segment);

        let leaf_did = leaf.skip_binder().def_id();
        if is_cgp_item(tcx, leaf_did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
            return Some(chain);
        }
        if is_descendable(tcx, leaf_did) {
            current = leaf.upcast(tcx);
            continue;
        }
        return None;
    }
    None
}

/// Solve `predicate` and return the failing leaf that leads toward the root cause, paired with
/// its cause-chain ancestors (leaf→root order). The leaf is chosen as the first fulfillment
/// error that is either a `HasField` or a descendable CGP wiring obligation.
fn solve_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    predicate: ty::Predicate<'tcx>,
) -> Option<(
    ty::PolyTraitPredicate<'tcx>,
    Vec<ty::PolyTraitPredicate<'tcx>>,
)> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new_with_diagnostics(&infcx);
    ocx.register_obligation(Obligation::new(
        tcx,
        ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        predicate,
    ));

    for err in ocx.evaluate_obligations_error_on_ambiguity() {
        let Some(leaf) = err.obligation.predicate.as_trait_clause() else {
            continue;
        };
        let did = leaf.skip_binder().def_id();
        if is_cgp_item(tcx, did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) || is_descendable(tcx, did) {
            return Some((leaf, walk_cause_parents(err.obligation.cause.code())));
        }
    }
    None
}

/// Collect the trait predicates on a failing obligation's derived-cause chain, from the
/// immediate parent up to the root obligation — the same "required for …" chain rustc renders
/// as notes, read here as typed predicates.
fn walk_cause_parents<'tcx>(
    mut code: &ObligationCauseCode<'tcx>,
) -> Vec<ty::PolyTraitPredicate<'tcx>> {
    let mut parents = Vec::new();
    loop {
        let (parent_pred, parent_code) = match code {
            ObligationCauseCode::ImplDerived(cause) => {
                (cause.derived.parent_trait_pred, &cause.derived.parent_code)
            }
            ObligationCauseCode::BuiltinDerived(derived)
            | ObligationCauseCode::WellFormedDerived(derived) => {
                (derived.parent_trait_pred, &derived.parent_code)
            }
            _ => break,
        };
        parents.push(parent_pred);
        code = parent_code;
    }
    parents
}

/// Whether a failing leaf trait is a CGP wiring obligation worth re-solving to descend one
/// dependency layer deeper — `IsProviderFor` or `CanUseComponent`, both defined by
/// `cgp-component`. An ordinary trait is deliberately excluded so the descent never strays
/// outside CGP's own machinery.
fn is_descendable(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    is_cgp_item(tcx, def_id, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, def_id, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
}

/// Fold the predicate chain into a rendered dependency tree, one node per meaningful step, with
/// each CGP wiring trait replaced by its human form (see [`label_for`]). The chain is linear, so
/// the tree is a single spine from the checked component down to the missing field.
fn build_tree<'tcx>(
    tcx: TyCtxt<'tcx>,
    chain: &[ty::PolyTraitPredicate<'tcx>],
    context: Ty<'tcx>,
    names: &ComponentNameMap,
) -> Option<DependencyTree> {
    let labels: Vec<String> = chain
        .iter()
        .filter_map(|pred| label_for(tcx, *pred, context, names))
        .collect();

    let mut rev = labels.into_iter().rev();
    let mut node = DependencyTree::leaf(rev.next()?);
    for label in rev {
        node = DependencyTree::node(label, vec![node]);
    }
    Some(node)
}

/// The human-readable label for one predicate in the dependency chain, replacing each CGP wiring
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
