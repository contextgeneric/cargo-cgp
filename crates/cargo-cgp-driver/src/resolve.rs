//! Typed root-cause resolution for CGP check-trait failures.
//!
//! This is the compiler-internals half of the diagnostic replacement. When the emitter sees a CGP
//! wiring failure, it asks this module to recover the *real* root cause(s) — and the transitive
//! dependency chain that leads to each — by walking the wiring's trait obligations rather than by
//! reading the rendered error text. Two entry points recover the starting obligation differently:
//! [`resolve_check_failure`] anchors on a `check_components!` entry (the common case, below), while
//! [`resolve_use_site`] handles a failure reported at a *use site* — a consumer-method call
//! (`E0599`) whose obligation no check impl carries — by recovering the context type from the
//! diagnostic's spans and re-checking every component that context wires. Both feed the same walk.
//!
//! The check-entry flow, all DefId-anchored to the CGP crates so a same-named type from elsewhere
//! can never drive it:
//!
//! 1. A `check_components!` entry expands to `impl __CheckCtx<Marker, Params> for Ctx {}`,
//!    whose check trait carries `CanUseComponent<Marker, Params>` as a supertrait. We find
//!    the impl whose `Self` type span equals the diagnostic's primary span — that is the
//!    entry the error is about — and instantiate the supertrait with the impl's trait ref to
//!    get the concrete obligation `Ctx: CanUseComponent<Marker, Params>`.
//! 2. From that obligation we walk *down* the dependency graph: for each failing obligation we
//!    find the impl that would satisfy it and take its `where`-clause obligations as the
//!    children, keeping only the ones that do not already hold. A branch descends only the CGP
//!    wiring vocabulary — `CanUseComponent`/`IsProviderFor`/`DelegateComponent`, provider traits,
//!    and obligations on the context itself — and stops at any *terminal* unmet bound: an unmet
//!    `cgp_field::HasField` (the field leaf) or an ordinary bound on a foreign type (`f64: Eq`).
//!    This descent unifies against candidate impls with `fresh_args_for_item`, rather than using
//!    `SelectionContext`, which asserts against the next-generation solver the driver runs under.
//! 3. This all runs *during* trait solving — the emitter reaches the live `TyCtxt` through
//!    `ty::tls` while a check error is being emitted — yet fresh inference contexts re-entered
//!    here solve cleanly; that re-entrancy is the load-bearing assumption behind the design.
//! 4. Each root-cause path is rendered as a `cargo tree`-style [`DependencyTree`] with every CGP
//!    wiring trait replaced by its human form (`CanUseComponent`→consumer-trait impl, `IsProviderFor`
//!    →provider-trait impl, `HasField`→field-trait impl), and a field name is decoded structurally
//!    from its `Symbol!`. A field leaf ([`Leaf::Field`]) drives a clean, tree-first replacement of
//!    the whole diagnostic; any other leaf ([`Leaf::Bound`]) keeps rustc's own header and only
//!    swaps the sub-notes for the tree. When no branch reaches a reportable leaf the resolver
//!    yields `None`, and the caller falls back to the untouched text-rewrite pipeline.
//!
//! For each leaf the resolver also inspects the *actual struct* the `HasField` bound lands on —
//! its own named fields, and, if the field is not there, the fields of the structs along its
//! `Deref` chain — so it can tell a genuinely absent field from one that is present but not derived.
//! A present field means that struct is missing its `#[derive(HasField)]`; the emitter then words
//! the diagnostic as an unimplemented `HasField` accessor and adds a `help` pointing at the derive
//! (see [`FieldIssue`]).
//!
//! Component markers are resolved to their consumer/provider trait names through the
//! [`ComponentNameMap`] keyed by each marker's *full path* (`def_path_str`), not its bare name, so
//! two components that share a name in different modules never collide.

use cargo_cgp_error_processing::ComponentTraitNames;
use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::tree::DependencyTree;
use rustc_hir::ItemKind;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::print::PrintTraitRefExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized, Upcast as _};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span};
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE,
    DELEGATE_COMPONENT_TRAIT, HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};

/// Why a required `HasField` bound is unmet — the distinction that tells a genuinely missing
/// field apart from one some struct actually carries but has not derived. CGP's `HasField` follows
/// `Deref` (a blanket impl forwards to the target), so a field on a `Deref` target resolves when
/// the target derives it; the failure the resolver diagnoses is a field present on some struct that
/// has no `HasField` impl for it. (A field present *with a mismatched type* keeps its `HasField`
/// trait impl and fails only the associated-type projection — an `E0271` the resolver leaves to
/// the fallback — so it never reaches this classification.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldIssue {
    /// No struct in the context's `Deref` chain carries a field of this name: it is genuinely
    /// missing and must be added.
    Missing,
    /// The context struct itself carries a field of this name, yet the `HasField` bound is unmet:
    /// the struct is missing (or has an incomplete) `#[derive(HasField)]` for it.
    Present,
    /// The context does not carry the field directly, but a struct reached through its `Deref`
    /// chain does. Since `HasField` follows `Deref`, the bound would hold if that target derived
    /// the field; the fault is that the target does not derive `HasField`.
    PresentViaDeref {
        /// The `Deref`-reachable struct that carries the field, e.g. `AppFields`.
        target: String,
    },
}

/// What a resolved dependency chain bottoms out on — the actual root cause the tree leads to.
pub enum Leaf {
    /// A `HasField` bound the wiring needs. The emitter renders this as a clean, tree-first
    /// diagnostic of its own, with a header worded by the [`FieldIssue`] and a derive `help`.
    Field {
        /// The field name, decoded from its `Symbol!`, e.g. `height`.
        name: String,
        /// The struct the `HasField` bound lands on — the type that must carry (or derive) the
        /// field. Usually the checked context, but a nested getter can make it another type.
        owner: String,
        /// Whether the field is genuinely missing, present-but-underived, or behind a `Deref`.
        issue: FieldIssue,
    },
    /// Any other terminal unmet bound — an ordinary trait bound (`f64: Eq`), an unmet abstract
    /// type, and so on. The emitter keeps rustc's own header for these and only replaces the
    /// sub-notes with the dependency tree.
    Bound {
        /// The bound restated as `self: Trait`, e.g. `f64: std::cmp::Eq`, for the note lead and
        /// for de-duplicating a leaf reached by several paths.
        summary: String,
    },
}

/// One recovered root cause: the leaf the chain bottoms out on and the transitive dependency
/// chain that leads to it, rendered as a single spine.
pub struct Cause {
    /// What the chain bottoms out on.
    pub leaf: Leaf,
    /// The dependency chain from the checked component down to the leaf.
    pub tree: DependencyTree,
}

/// The recovered root cause(s) of a check failure, in owned form so they outlive the inference
/// contexts they were read from.
pub struct Resolved {
    /// The checked context type, e.g. `Rectangle`.
    pub context: String,
    /// One entry per distinct root cause, in first-seen order.
    pub causes: Vec<Cause>,
}

impl Cause {
    /// A stable key that de-duplicates a leaf reached by several dependency paths — the field name
    /// for a field, the bound restatement otherwise.
    fn key(&self) -> &str {
        match &self.leaf {
            Leaf::Field { name, .. } => name,
            Leaf::Bound { summary } => summary,
        }
    }
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
) -> Option<Resolved> {
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

            let Some(top) = concrete.as_trait_clause() else {
                continue;
            };
            if let Some(resolved) = resolve_leaves(tcx, top, names) {
                return Some(resolved);
            }
        }
    }
    None
}

/// Resolve the root cause(s) of a CGP wiring failure reported at a *use site* rather than a
/// `check_components!` entry — a consumer-method call (`E0599`) or any other diagnostic whose
/// obligation is not recoverable from a check impl. There is no check impl to anchor on, so the
/// context type is recovered from a diagnostic span that lands on a local struct/enum definition,
/// and every component that context wires (through its `DelegateComponent` impls) is re-checked;
/// each one that cannot be used contributes its dependency tree. `None` when no context is found
/// or no wired component fails resolvably.
pub fn resolve_use_site(
    tcx: TyCtxt<'_>,
    spans: &[Span],
    names: &ComponentNameMap,
) -> Option<Resolved> {
    let can_use_did = find_cgp_trait(tcx, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;

    // A diagnostic span can land on a provider struct as well as the real context (both are local
    // ADTs), so try each candidate and keep the first that actually wires a failing component.
    for context in context_candidates_from_spans(tcx, spans) {
        let mut causes: Vec<Cause> = Vec::new();
        for marker in delegated_markers(tcx, context) {
            // `Ctx: CanUseComponent<Marker, ()>` — the parameterless form, which suits the
            // components a use-site failure exercises; a component whose `()` form holds is skipped.
            let trait_ref = ty::TraitRef::new(tcx, can_use_did, [context, marker, tcx.types.unit]);
            let top: ty::PolyTraitPredicate<'_> = ty::Binder::dummy(trait_ref).upcast(tcx);
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, top, names) {
                for cause in resolved.causes {
                    if !causes.iter().any(|c| c.key() == cause.key()) {
                        causes.push(cause);
                    }
                }
            }
        }
        if !causes.is_empty() {
            return Some(Resolved {
                context: tcx.erase_and_anonymize_regions(context).to_string(),
                causes,
            });
        }
    }
    None
}

/// The candidate context types of a use-site failure: every local struct or enum whose definition
/// span contains one of the diagnostic's spans — for an `E0599` method error that includes the
/// "method not found for this struct" span on the receiver's type. Each ADT is returned with
/// identity arguments (so a generic context keeps its generic form); the caller picks the one that
/// actually wires a failing component, which discards a provider struct that merely shares a span.
fn context_candidates_from_spans<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<Ty<'tcx>> {
    let mut candidates = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        let did = local.to_def_id();
        if !matches!(
            tcx.def_kind(did),
            rustc_hir::def::DefKind::Struct | rustc_hir::def::DefKind::Enum
        ) {
            continue;
        }
        let def_span = tcx.def_span(did);
        if spans.iter().any(|&span| def_span.contains(span)) {
            candidates.push(tcx.type_of(did).instantiate_identity().skip_norm_wip());
        }
    }
    candidates
}

/// The component markers a context wires, read from its `DelegateComponent<Marker>` impls — the
/// components whose use-site failure the resolver re-checks.
fn delegated_markers<'tcx>(tcx: TyCtxt<'tcx>, context: Ty<'tcx>) -> Vec<Ty<'tcx>> {
    let Some(delegate_did) = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
    else {
        return Vec::new();
    };
    let context = tcx.erase_and_anonymize_regions(context);

    let mut markers = Vec::new();
    for impl_did in tcx.all_impls(delegate_did) {
        let impl_self = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        if tcx.erase_and_anonymize_regions(impl_self) != context {
            continue;
        }
        // `DelegateComponent<Marker>` — args are `[Self, Marker]`.
        let marker = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip()
            .args
            .type_at(1);
        markers.push(tcx.erase_and_anonymize_regions(marker));
    }
    markers
}

/// The `DefId` of the CGP trait named `name` defined by crate `krate`, or `None` if the crate does
/// not use CGP. Anchored by name *and* crate, like every other CGP lookup here.
fn find_cgp_trait(tcx: TyCtxt<'_>, name: &str, krate: &str) -> Option<DefId> {
    tcx.all_traits_including_private()
        .find(|&did| is_cgp_item(tcx, did, name, krate))
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

/// Walk the dependency graph of `top` (`Ctx: CanUseComponent<Marker, Params>`) and, for each
/// distinct terminal unmet bound it bottoms out on, return that leaf with its rendered dependency
/// chain. `None` when no branch reaches a resolvable leaf.
fn resolve_leaves<'tcx>(
    tcx: TyCtxt<'tcx>,
    top: ty::PolyTraitPredicate<'tcx>,
    names: &ComponentNameMap,
) -> Option<Resolved> {
    let context = tcx.erase_and_anonymize_regions(top.skip_binder().self_ty());

    let mut causes: Vec<Cause> = Vec::new();
    for path in collect_leaf_paths(tcx, top, context, &[], 0) {
        let leaf_ref = path.last()?.skip_binder().trait_ref;
        // A path that bottoms out on pure wiring plumbing (a routing dead-end) is not a root
        // cause — a real cause is found down another branch — so drop it rather than report it.
        if !is_reportable_leaf(tcx, leaf_ref) {
            continue;
        }
        let leaf = classify_leaf(tcx, leaf_ref);
        // One sub-error per distinct leaf: a leaf wanted by several branches is one fix.
        if causes.iter().any(|c| c.key() == leaf_key(&leaf)) {
            continue;
        }
        let labels: Vec<String> = path
            .iter()
            .filter_map(|pred| label_for(tcx, *pred, context, names))
            .collect();
        if let Some(tree) = spine(labels) {
            causes.push(Cause { leaf, tree });
        }
    }

    if causes.is_empty() {
        return None;
    }
    Some(Resolved {
        context: context.to_string(),
        causes,
    })
}

/// Classify the terminal predicate a dependency chain bottoms out on. A `HasField` becomes a
/// [`Leaf::Field`] (inspecting the struct so the emitter can tell missing from underived); any
/// other bound becomes a [`Leaf::Bound`] restating it as `self: Trait`.
fn classify_leaf<'tcx>(tcx: TyCtxt<'tcx>, leaf_ref: ty::TraitRef<'tcx>) -> Leaf {
    if is_cgp_item(tcx, leaf_ref.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE)
        && let Some(name) = decode_symbol(tcx, leaf_ref.args.type_at(1))
    {
        let owner = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        let issue = field_issue(tcx, owner, &name);
        return Leaf::Field {
            name,
            owner: owner.to_string(),
            issue,
        };
    }
    Leaf::Bound {
        summary: format!(
            "{}: {}",
            leaf_ref.self_ty(),
            leaf_ref.print_only_trait_path()
        ),
    }
}

/// Whether a terminal leaf is a real root cause worth reporting, rather than pure wiring plumbing.
/// A `CanUseComponent`, `IsProviderFor`, or `DelegateComponent` that bottoms out unmet is a routing
/// dead-end (the real cause sits down another branch), so it is dropped instead of shown.
fn is_reportable_leaf<'tcx>(tcx: TyCtxt<'tcx>, leaf_ref: ty::TraitRef<'tcx>) -> bool {
    let did = leaf_ref.def_id;
    !is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        && !is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        && !is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
}

/// The de-duplication key for a freshly classified leaf, mirroring [`Cause::key`].
fn leaf_key(leaf: &Leaf) -> &str {
    match leaf {
        Leaf::Field { name, .. } => name,
        Leaf::Bound { summary } => summary,
    }
}

/// Collect every root→leaf path that bottoms out on a terminal unmet bound, by descending the
/// failing obligation's dependency graph. `pred` is a failing obligation and `prefix` is the path
/// of predicates above it. A `HasField` completes a path directly; any other obligation contributes
/// the `where`-clause obligations of the impl that would satisfy it, recursing into just the ones
/// that do not already hold. An obligation with **no** satisfying impl is itself a terminal leaf
/// (an ordinary bound like `f64: Eq`). Following *every* unmet dependency (not one) is what surfaces
/// independent causes as separate paths. Bounded by [`MAX_DEPTH`].
///
/// One case yields no path on purpose: an unmet obligation whose satisfying impl's `where`-clause
/// obligations all *hold*. The obligation is then unmet for a reason the trait-clause walk cannot
/// see — a projection/associated-type mismatch, e.g. a field present with the wrong type — so
/// rendering a tree here would point at a leaf that is not the real fault. Declining lets the
/// caller keep rustc's own (already precise) diagnostic for that case.
///
/// The descent stops at any ordinary bound on a foreign type (a bound whose `Self` is not the
/// context and whose trait is not CGP wiring), treating it as the terminal leaf. That bound *is*
/// the root cause a reader wants (`f64: Eq`); descending it would wander into whatever unrelated
/// `std` blanket impl happens to match its `Self` (e.g. `impl<F: FnPtr> Eq for F`) and fabricate a
/// misleading chain.
fn collect_leaf_paths<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
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

    if !is_descendable(tcx, pred, context) {
        // An ordinary bound on a foreign type — the root-cause leaf. Do not walk into `std` impls.
        return vec![path];
    }

    let Some(children) = impl_where_obligations(tcx, pred) else {
        // No impl satisfies `pred` at all — `pred` is itself the terminal root-cause bound.
        return vec![path];
    };

    let unmet: Vec<_> = children
        .into_iter()
        .filter(|nested| !holds(tcx, *nested))
        .collect();
    if unmet.is_empty() {
        // Matched an impl, yet every `where`-clause holds: the fault is a projection the walk
        // cannot see. Yield nothing so the resolver declines to this branch.
        return Vec::new();
    }

    let mut paths = Vec::new();
    for nested in unmet {
        paths.extend(collect_leaf_paths(tcx, nested, context, &path, depth + 1));
    }
    paths
}

/// Whether the descent should walk *into* `pred`'s dependencies, rather than treat `pred` as a
/// terminal leaf. It descends the CGP wiring vocabulary (`CanUseComponent`, `IsProviderFor`,
/// `DelegateComponent`), any provider trait (a `ProvideFoo: Foo<App>` bound routes on to the
/// provider's own dependencies), and any obligation on the context itself (its getter and
/// capability traits). It stops at everything else — an ordinary bound like `f64: Eq`, whose `Self`
/// is a foreign type, is a leaf, not a step to descend.
fn is_descendable<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;
    tcx.erase_and_anonymize_regions(trait_ref.self_ty()) == context
        || is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
}

/// The instantiated `where`-clause trait obligations of the impl that would satisfy `obligation`
/// — its direct dependencies — or `None` when no impl matches at all (so the caller can treat the
/// obligation as a terminal leaf). Found by unifying `obligation` with each candidate impl's trait
/// ref (the next-solver-safe `fresh_args_for_item` + `eq` dance `SelectionContext` is unavailable
/// for), then instantiating and normalizing that impl's predicates. `Some(vec![])` means an impl
/// matched but carries no trait-clause `where` obligations.
fn impl_where_obligations<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
) -> Option<Vec<ty::PolyTraitPredicate<'tcx>>> {
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
        return Some(obligations);
    }
    None
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

/// Bound on how far the `Deref` chain is followed when looking for a field, so a cyclic `Deref`
/// (`A: Deref<Target = B>`, `B: Deref<Target = A>`) cannot make the search loop.
const MAX_DEREF: u32 = 16;

/// Classify why the `HasField` bound on `owner` for `field` is unmet: whether `owner` genuinely
/// lacks the field, carries it directly (so only the `HasField` impl or its type is at fault), or
/// reaches it only through a `Deref` target the derive does not cross.
fn field_issue<'tcx>(tcx: TyCtxt<'tcx>, owner: Ty<'tcx>, field: &str) -> FieldIssue {
    if adt_has_field(owner, field) {
        return FieldIssue::Present;
    }
    let mut current = owner;
    for _ in 0..MAX_DEREF {
        let Some(target) = deref_target(tcx, current) else {
            break;
        };
        if adt_has_field(target, field) {
            return FieldIssue::PresentViaDeref {
                target: target.to_string(),
            };
        }
        current = target;
    }
    FieldIssue::Missing
}

/// Whether `ty` is a struct with a named field called `field`. Only a struct can carry named
/// fields a `HasField` derive would key on, so an enum, tuple, or non-ADT is never a match.
fn adt_has_field(ty: Ty<'_>, field: &str) -> bool {
    match ty.kind() {
        ty::Adt(def, _) if def.is_struct() => def
            .non_enum_variant()
            .fields
            .iter()
            .any(|f| f.name.as_str() == field),
        _ => false,
    }
}

/// The `Deref::Target` of `ty`, read straight from the concrete `impl Deref for ty` rather than by
/// normalizing a projection, so it needs no inference context. Returns `None` when `ty` has no
/// matching `Deref` impl. Matches the impl by its `Self` type, so a generic `Deref` impl whose
/// `Self` is not exactly `ty` is skipped — sufficient for the concrete contexts a check targets.
fn deref_target<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let deref_trait = tcx.lang_items().deref_trait()?;
    let ty = tcx.erase_and_anonymize_regions(ty);

    for impl_did in tcx.all_impls(deref_trait) {
        let impl_self = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        if tcx.erase_and_anonymize_regions(impl_self) != ty {
            continue;
        }
        // The `Deref` impl's single associated type is its `Target`; its value is the target type.
        for assoc in tcx.associated_items(impl_did).in_definition_order() {
            if assoc.kind.tag() == ty::AssocTag::Type {
                let target = tcx
                    .type_of(assoc.def_id)
                    .instantiate_identity()
                    .skip_norm_wip();
                return Some(tcx.erase_and_anonymize_regions(target));
            }
        }
    }
    None
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
/// trait with the concept it stands for: `CanUseComponent` with the consumer-trait impl,
/// `IsProviderFor` with the provider-trait impl (its provider trait, context, and provider struct),
/// `HasField` with the field-trait impl (the field and the struct that must carry it). Any other
/// trait — a user's own consumer or getter capability — is shown as a trait impl for its self type.
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
        // `CanUseComponent<Marker, Params>` — the component's extra parameters, reattached so a
        // generic component's consumer trait reads as written (`CanCalculateArea<u32, u64, bool>`).
        let generics = render_params(trait_ref.args.type_at(2));
        Some(format!(
            "consumer trait impl `{consumer}{generics}` for context `{}`",
            trait_ref.self_ty()
        ))
    } else if is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE) {
        // Drop the routing `IsProviderFor` for the context itself; keep the real providers.
        if trait_ref.self_ty() == context {
            return None;
        }
        let provider = trait_ref.self_ty().to_string();
        let provider_trait = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.provider);
        // `IsProviderFor<Provider, Marker, Context, Params>` — the third argument is the context,
        // the fourth the component's extra parameters (reattached to the provider trait).
        let provider_context = trait_ref.args.type_at(2);
        let generics = render_params(trait_ref.args.type_at(3));
        Some(format!(
            "provider trait impl `{provider_trait}{generics}` with context `{provider_context}` for provider `{provider}`"
        ))
    } else if is_cgp_item(tcx, did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
        let field = decode_symbol(tcx, trait_ref.args.type_at(1))?;
        Some(format!(
            "field trait impl `HasField` with field `{field}` for `{}`",
            trait_ref.self_ty()
        ))
    } else if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
    {
        None
    } else {
        Some(format!(
            "trait impl `{}` for `{}`",
            tcx.item_name(did),
            trait_ref.self_ty()
        ))
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

/// Render a component's extra type parameters as a trait generic list — `<u32, u64, bool>`, or the
/// empty string when there are none. CGP groups the parameters into the `Params` slot of
/// `CanUseComponent`/`IsProviderFor`: none as the unit `()`, a single one bare, several as a tuple,
/// so a tuple is unwrapped and a bare parameter reattached directly.
fn render_params(params: Ty<'_>) -> String {
    match params.kind() {
        ty::Tuple(elems) if elems.is_empty() => String::new(),
        ty::Tuple(elems) => format!(
            "<{}>",
            elems
                .iter()
                .map(|elem| elem.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => format!("<{params}>"),
    }
}

/// Resolve a component marker type to its consumer or provider trait name through the name map,
/// keyed by the marker's *full path* so two same-named markers in different modules never collide.
/// Falls back to the marker's bare item name when the component is not in the map, and to the
/// marker's printed form when it is not an ADT at all.
fn marker_role(
    tcx: TyCtxt<'_>,
    marker: Ty<'_>,
    names: &ComponentNameMap,
    role: impl Fn(ComponentTraitNames) -> String,
) -> String {
    let ty::Adt(def, _) = marker.kind() else {
        return marker.to_string();
    };
    match names.get_by_path(&tcx.def_path_str(def.did())) {
        Some(entry) => role(entry),
        None => tcx.item_name(def.did()).to_string(),
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
