//! Recovering the starting obligation of a check failure.
//!
//! Four entry points recover the obligation differently, then feed the same
//! [walk](crate::resolve::walk): [`resolve_check_failure`] anchors on a `check_components!` entry
//! (by matching the failing diagnostic's caret to the check impl's `Self`-type span);
//! [`resolve_impl_site`] handles a wiring failure surfaced *inside a hand-written `impl Trait for
//! Context` block* (by recovering the exact failing obligation — with its concrete component
//! parameters — from the impl's CGP consumer supertrait); [`resolve_wrapper_chain`] handles the same
//! shape when the impl's `Self` is a *foreign* wrapper holding the context (by descending its
//! supertrait's ordinary `where`-clause hops to a CGP consumer on the context, the routing-glue
//! case); and [`resolve_use_site`] handles a
//! consumer-method `E0599` (by recovering the context ADT from the diagnostic's spans and
//! re-checking the parameterless form of every component that context wires).

use cargo_cgp_error_processing::code::DEP_TRAIT_IMPL;
use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::tree::DependencyTree;
use cargo_cgp_error_processing::{Cause, Resolved};
use rustc_hir::ItemKind;
use rustc_hir::def::DefKind;
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::print::PrintTraitRefExt as _;
use rustc_middle::ty::{
    self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode, Unnormalized, Upcast as _,
};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::cgp_item::{find_cgp_trait, is_cgp_item, is_provider_trait};
use crate::resolve::walk::{holds, resolve_leaves};

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

/// Resolve the root cause(s) of a CGP wiring failure reported *inside a hand-written `impl Trait
/// for Context` block* — the shape a wrapper trait that carries a CGP consumer trait as a
/// supertrait produces when it is implemented directly on a concrete context (the transfer
/// example's per-endpoint `impl CanHandleApiSend<Api> for MockApp`, added to bound a future
/// `Send`). Such a failure anchors on neither a `check_components!` entry nor a consumer-method
/// call, and its caret sits on the impl rather than on the context's own type definition, so
/// [`resolve_use_site`] cannot recover the context from a struct-definition span.
///
/// This entry recovers the context from the enclosing impl's `Self` type, and — crucially — the
/// *exact* failing obligation from the impl's CGP consumer supertrait, so a generic component
/// carries its concrete parameter (`CanCalculateArea<Rectangle>`, not the `()` form the use-site
/// re-check would substitute). It reconstructs the `Ctx: CanUseComponent<Marker, Params>`
/// obligation that supertrait stands for and walks it exactly as a check entry would. The recovered
/// tree is then headed by the impl's *own* trait — the wrapper the programmer wrote — so the
/// diagnostic points at their code and the CGP consumer it reduces to follows beneath: the failure
/// reads `CanHandleApiSend → CanHandleApi → …`, not `CanHandleApi → …` with the wrapper dropped.
/// Because the wrapper is a distinct trait from that supertrait, its error stands on its own rather
/// than de-duplicating into the `check_components!` entry. `None` when no enclosing impl on a local
/// context carries an unmet, reconstructable CGP consumer supertrait.
pub fn resolve_impl_site(
    tcx: TyCtxt<'_>,
    spans: &[Span],
    names: &ComponentNameMap,
) -> Option<Resolved> {
    for impl_did in enclosing_trait_impls(tcx, spans) {
        // Safe because `enclosing_trait_impls` keeps only `of_trait` impls.
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
        // Only a local struct/enum is a context whose wiring we can re-check; skip an impl on a
        // foreign type (e.g. `impl … for Router<Arc<App>>`) or a type parameter. Such a foreign
        // wrapper — where the CGP consumer failure sits several `where`-clause hops down rather than
        // in a direct supertrait — is [`resolve_wrapper_chain`]'s job.
        if !is_local_adt(context) {
            continue;
        }

        // The error the programmer actually wrote is the impl's *own* trait — the wrapper (e.g.
        // `CanHandleApiSend<Api>`), not the CGP consumer supertrait it reduces to. That original
        // obligation heads the dependency tree and names the diagnostic, so the reader sees the
        // failure at their own code; the CGP consumer it depends on follows as the next node. Being
        // a distinct trait from that supertrait, the wrapper's error is reported on its own rather
        // than de-duplicated into the `check_components!` entry for the supertrait.
        let obligation: ty::PolyTraitPredicate<'_> = ty::Binder::dummy(trait_ref).upcast(tcx);
        if let Some((consumers_are_cgp, causes)) = wrapper_consumer_causes(tcx, obligation, names) {
            return Some(Resolved {
                context: context.to_string(),
                consumers: vec![trait_ref.print_only_trait_path().to_string()],
                consumers_are_cgp,
                // The wrapper is implemented directly on the context, so the subject is the context.
                subject_is_context: true,
                causes,
            });
        }
    }
    None
}

/// Resolve the root cause(s) of a CGP wiring failure reported *inside a hand-written `impl Trait for
/// Foreign` block whose `Self` is a foreign type holding the context* — the transfer example's
/// `impl CanAddApiRoutes for Router<Arc<MockApp>>`, where the routing wrapper's supertrait descends
/// through a chain of ordinary user-trait `where`-clauses (`… CanAddRoute<MockApp, …>` → `MockApp:
/// CanHandleApiSend<…>`) before it reaches a CGP consumer on the context. The context appears only
/// as a type *argument* of the failing traits, never as the impl's `Self`, so
/// [`resolve_impl_site`]'s "direct supertrait on a local context" recovery cannot fire.
///
/// This entry starts from the enclosing impl's own unmet supertrait and walks *down* the ordinary
/// trait obligations — via each impl's `where`-clauses — until one lands on a CGP consumer (or a
/// wrapper carrying one) whose `Self` *is* a local context, at which point it hands off to the same
/// [`wrapper_consumer_causes`] recovery [`resolve_impl_site`] uses. Every ordinary hop between the
/// impl and that handoff becomes a `trait impl` node, so the tree reads from the code the programmer
/// wrote (`CanAddApiRoutes → CanAddMainApiRoutes → CanAddRoute → CanHandleApiSend → CanHandleApi →
/// …`) down to the root cause. Because it *re-evaluates* each obligation with the trait solver rather
/// than trusting rustc's cascade-suppressed diagnostic, it recovers the cause even where rustc's own
/// error names only the outermost unsatisfied bound. `None` when no enclosing impl's supertrait
/// chain reaches a CGP consumer on a local context.
pub fn resolve_wrapper_chain(
    tcx: TyCtxt<'_>,
    spans: &[Span],
    names: &ComponentNameMap,
) -> Option<Resolved> {
    for impl_did in enclosing_trait_impls(tcx, spans) {
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        let self_ty = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
        // The direct-supertrait, local-`Self` case is [`resolve_impl_site`]'s (tried first); here we
        // handle the rest, where the CGP consumer is reached only through `where`-clause hops.
        let wrapper = trait_ref.print_only_trait_path().to_string();
        let top_node = format!("[{DEP_TRAIT_IMPL}] trait impl `{wrapper}` for `{self_ty}`");

        // Descend each unmet supertrait of the impl's own trait, collecting a cause per CGP handoff
        // reached beneath it.
        let mut causes: Vec<Cause> = Vec::new();
        for &(clause, _) in tcx
            .explicit_super_predicates_of(trait_ref.def_id)
            .skip_binder()
        {
            let concrete = clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));
            let Some(sup) = concrete.as_trait_clause() else {
                continue;
            };
            if holds(tcx, sup) {
                continue;
            }
            collect_wrapper_chain_causes(tcx, sup, names, &[], 0, &mut causes);
        }

        if !causes.is_empty() {
            // Head every cause's tree with the impl's own trait — the code the programmer wrote.
            for cause in &mut causes {
                let tree = std::mem::replace(&mut cause.tree, DependencyTree::leaf(String::new()));
                cause.tree = DependencyTree::node(top_node.clone(), vec![tree]);
            }
            return Some(Resolved {
                context: self_ty.to_string(),
                consumers: vec![wrapper],
                // The impl's own trait heads the header; a routing wrapper such as `CanAddApiRoutes`
                // is a plain trait, not a CGP consumer, so it reads `the trait` (`CGP-E009`).
                consumers_are_cgp: consumer_provider_trait(tcx, trait_ref.def_id).is_some(),
                // `Self` is a foreign wrapper holding the context, not the context itself — reached
                // here only *because* it is not the context (else `resolve_impl_site` caught it).
                subject_is_context: false,
                causes,
            });
        }
    }
    None
}

/// Bound on how deep the wrapper-chain descent walks before giving up, matching the dependency-graph
/// walk's own bound. Real wrapper chains (`CanAddApiRoutes → CanAddMainApiRoutes → CanAddRoute → …`)
/// are only a few hops.
const MAX_WRAPPER_DEPTH: u32 = 32;

/// Descend `obligation` — an unmet ordinary-trait bound — looking for the nearest CGP consumer on a
/// local context beneath it, and append a cause per one found to `out`. Such a consumer obligation
/// (`App: CanHandleApi<GreetApi>`) is the handoff: [`consumer_handoff_causes`] recovers its
/// dependency tree, the descent stops there (that recovery walks the CGP chain the rest of the way),
/// and the ordinary hops that led here are prepended as `trait impl` nodes. Anything else is
/// descended through its satisfying impl's `where`-clause obligations ([`wrapper_chain_children`]),
/// following only the unmet ones.
///
/// The descent reaches the consumer two ways, both handled by `wrapper_chain_children`: as a direct
/// `where`-clause bound, or — the transfer example's case — as the *base* of a projection bound
/// (`<App as CanHandleApi<GreetApi>>::Response: Send`), since a `where`-clause naming an associated
/// type of the broken consumer is what makes the outer trait genuinely fail (the direct
/// `App: CanHandleApiSend<GreetApi>` bound is instead *assumed to hold* off its ill-formed impl, so
/// it is never the failing route). A branch with no satisfying impl, or one reaching no consumer
/// within [`MAX_WRAPPER_DEPTH`], contributes nothing — only a genuine CGP consumer is ever reported,
/// so a wandering descent cannot fabricate a cause.
fn collect_wrapper_chain_causes<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
    names: &ComponentNameMap,
    chain: &[String],
    depth: u32,
    out: &mut Vec<Cause>,
) {
    if depth > MAX_WRAPPER_DEPTH {
        return;
    }

    // The handoff: `obligation` is a CGP consumer on a local context. Recover its cause tree and
    // prepend the chain of ordinary hops that led here.
    if let Some(causes) = consumer_handoff_causes(tcx, obligation, names) {
        for cause in causes {
            if out.iter().any(|c| c.key() == cause.key()) {
                continue;
            }
            out.push(Cause {
                leaf: cause.leaf,
                tree: wrap_with_chain(chain, cause.tree),
            });
        }
        return;
    }

    // Not a handoff: descend the obligations of the impl that would satisfy it — its direct trait
    // bounds and the base trait ref of each associated-type `where`-clause bound.
    let Some(children) = wrapper_chain_children(tcx, obligation) else {
        return;
    };
    let trait_ref = obligation.skip_binder().trait_ref;
    let node = format!(
        "[{DEP_TRAIT_IMPL}] trait impl `{}` for `{}`",
        trait_ref.print_only_trait_path(),
        tcx.erase_and_anonymize_regions(trait_ref.self_ty()),
    );
    let mut next_chain = chain.to_vec();
    next_chain.push(node);
    for child in children {
        if holds(tcx, child) {
            continue;
        }
        collect_wrapper_chain_causes(tcx, child, names, &next_chain, depth + 1, out);
    }
}

/// Recover a cause per root cause of `obligation` when it *is* a CGP consumer trait on a local
/// context (`App: CanHandleApi<GreetApi>`) — the handoff the wrapper-chain descent hands off to the
/// ordinary [walk](crate::resolve::walk). It reconstructs the `CanUseComponent` obligation the
/// consumer stands for (with its concrete component parameters preserved) and walks it exactly as a
/// check entry would, so the returned trees are already headed by the consumer node. `None` when
/// `obligation`'s `Self` is not a local ADT or its trait is not a CGP consumer — so the descent
/// keeps walking rather than stopping on an ordinary bound.
fn consumer_handoff_causes<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
    names: &ComponentNameMap,
) -> Option<Vec<Cause>> {
    let trait_ref = obligation.skip_binder().trait_ref;
    let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
    if !is_local_adt(context) {
        return None;
    }
    // A CGP consumer trait pairs with a provider trait through its blanket impl; a plain trait does
    // not, so it is not a handoff.
    consumer_provider_trait(tcx, trait_ref.def_id)?;
    let top = consumer_can_use_obligation(tcx, context, obligation)?;
    let resolved = resolve_leaves(tcx, top, names)?;
    Some(resolved.causes)
}

/// The obligations to descend from an unmet ordinary-trait bound: the `where`-clause obligations of
/// the impl that would satisfy it, with each associated-type bound replaced by its *base* trait ref
/// (`<App as CanHandleApi<Api>>::Response: Send` → `App: CanHandleApi<Api>`). The base is what a
/// projection bound really rests on, and it is concrete even when the projected type itself is not —
/// which is exactly why the ordinary [`impl_where_obligations`] (which normalizes, turning the
/// projection into an inference variable it then drops) cannot surface it. `None` when no impl
/// matches. Mirrors [`impl_where_obligations`]'s next-solver-safe impl match, but reads the impl's
/// predicates un-normalized so an associated-type `Self` survives long enough to read its base.
fn wrapper_chain_children<'tcx>(
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

        let mut children = Vec::new();
        for (predicate, _) in tcx.predicates_of(impl_did).instantiate(tcx, impl_args) {
            // Keep the predicate un-normalized so an associated-type `Self` survives; `skip_norm_wip`
            // unwraps without normalizing, unlike the `ocx.normalize` a trait-clause walk uses.
            let clause = infcx.resolve_vars_if_possible(predicate.skip_norm_wip());
            let clause = tcx.erase_and_anonymize_regions(clause);
            let Some(tp) = clause.as_trait_clause() else {
                continue;
            };
            // An associated-type bound (`<App as CanHandleApi<Api>>::Response: Send`) descends to the
            // projection's base trait (`App: CanHandleApi<Api>`) — what it truly rests on — rather
            // than the bound itself.
            let child = match tp.self_ty().skip_binder().kind() {
                ty::Alias(_, alias) if matches!(alias.kind, ty::AliasTyKind::Projection { .. }) => {
                    ty::Binder::dummy(alias.trait_ref(tcx)).upcast(tcx)
                }
                _ => tp,
            };
            // An unconstrained impl parameter leaves inference vars behind; such a child cannot be
            // re-checked in a fresh context, so skip it.
            if child.has_non_region_infer() {
                continue;
            }
            children.push(child);
        }
        return Some(children);
    }
    None
}

/// Nest a top-to-bottom list of tree-node labels above `inner`, so `chain[0]` is the outermost node
/// and `inner` the innermost child.
fn wrap_with_chain(chain: &[String], inner: DependencyTree) -> DependencyTree {
    let mut tree = inner;
    for label in chain.iter().rev() {
        tree = DependencyTree::node(label.clone(), vec![tree]);
    }
    tree
}

/// Recover a cause per unmet CGP consumer supertrait of a wrapper obligation `Self: Wrapper` whose
/// `Self` is a local context, each tree headed by the wrapper's own `trait impl` node — the shared
/// heart of both [`resolve_impl_site`] (where the wrapper is the enclosing impl's own trait) and the
/// [`resolve_wrapper_chain`] handoff (where it is reached down a `where`-clause chain). Returns
/// whether the wrapper is itself a CGP consumer (for the header wording) and the causes, or `None`
/// when `Self` is not a local ADT or no CGP consumer supertrait is unmet — so a plain bound like
/// `Send`, or a wrapper on a foreign type, is skipped rather than mis-recovered.
fn wrapper_consumer_causes<'tcx>(
    tcx: TyCtxt<'tcx>,
    obligation: ty::PolyTraitPredicate<'tcx>,
    names: &ComponentNameMap,
) -> Option<(bool, Vec<Cause>)> {
    let trait_ref = obligation.skip_binder().trait_ref;
    let context = tcx.erase_and_anonymize_regions(trait_ref.self_ty());
    if !is_local_adt(context) {
        return None;
    }
    let wrapper = trait_ref.print_only_trait_path().to_string();
    let wrapper_node = format!("[{DEP_TRAIT_IMPL}] trait impl `{wrapper}` for `{context}`");

    let mut causes: Vec<Cause> = Vec::new();
    // Each supertrait the wrapper trait carries, instantiated for this `Self`. A CGP consumer trait
    // among them that does not hold is the wiring failure the wrapper surfaces.
    for &(clause, _) in tcx
        .explicit_super_predicates_of(trait_ref.def_id)
        .skip_binder()
    {
        let concrete = clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));
        let Some(sup) = concrete.as_trait_clause() else {
            continue;
        };
        if tcx.erase_and_anonymize_regions(sup.skip_binder().self_ty()) != context {
            continue;
        }
        if holds(tcx, sup) {
            continue;
        }
        let Some(top) = consumer_can_use_obligation(tcx, context, sup) else {
            continue;
        };
        let Some(resolved) = resolve_leaves(tcx, top, names) else {
            continue;
        };
        for cause in resolved.causes {
            if !causes.iter().any(|c| c.key() == cause.key()) {
                // Prepend the original wrapper obligation as the tree's top node, above the CGP
                // consumer chain the walk recovered.
                causes.push(Cause {
                    leaf: cause.leaf,
                    tree: DependencyTree::node(wrapper_node.clone(), vec![cause.tree]),
                });
            }
        }
    }

    if causes.is_empty() {
        return None;
    }
    // Whether the wrapper trait is a CGP *consumer* trait or a plain wrapper, decided by its
    // fingerprint: a consumer trait carries a blanket impl routing to a provider trait
    // (`consumer_provider_trait`), while a hand-written wrapper like `CanHandleApiSend` has only its
    // concrete impl. This picks `the consumer trait` vs `the trait` in the header.
    let consumers_are_cgp = consumer_provider_trait(tcx, trait_ref.def_id).is_some();
    Some((consumers_are_cgp, causes))
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
        let mut consumers: Vec<String> = Vec::new();
        for marker in delegated_markers(tcx, context) {
            // `Ctx: CanUseComponent<Marker, ()>` — the parameterless form, which suits the
            // components a use-site failure exercises; a component whose `()` form holds is skipped.
            let trait_ref = ty::TraitRef::new(tcx, can_use_did, [context, marker, tcx.types.unit]);
            let top: ty::PolyTraitPredicate<'_> = ty::Binder::dummy(trait_ref).upcast(tcx);
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, top, names) {
                for consumer in resolved.consumers {
                    if !consumers.contains(&consumer) {
                        consumers.push(consumer);
                    }
                }
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
                consumers,
                // A use-site failure recovers CGP consumer traits from the context's wired markers.
                consumers_are_cgp: true,
                // The subject is the checked context itself.
                subject_is_context: true,
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

/// The local trait-impl blocks (`impl Trait for Ty { … }`) whose source span contains one of the
/// diagnostic's spans — the impls a wiring failure can surface inside, whether the caret lands on
/// the impl header or deep in a method body. A trait impl (not an inherent one) is required
/// because the recovery reads the impl's trait supertraits. The *full* item span is taken from HIR
/// rather than `def_span`, which for an impl covers only the header and would miss a caret inside
/// the body (a forwarding `self.method(..)` call, say).
fn enclosing_trait_impls(tcx: TyCtxt<'_>, spans: &[Span]) -> Vec<DefId> {
    let mut impls = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let impl_span = tcx.hir_expect_item(local).span;
        if spans.iter().any(|&span| impl_span.contains(span)) {
            impls.push(local.to_def_id());
        }
    }
    impls
}

/// Whether `ty` is a struct or enum defined in the crate being compiled — the only kind of type
/// whose wiring the resolver re-checks as a context.
fn is_local_adt(ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if def.did().is_local())
}

/// Reconstruct the `Ctx: CanUseComponent<Marker, Params>` obligation a CGP consumer-trait bound
/// (`Ctx: CanCalculateArea<Rectangle>`) stands for, so it can be walked exactly as a
/// `check_components!` entry is — with the component's concrete parameters preserved. Returns
/// `None` when `consumer` is not a recognizable CGP consumer trait (its blanket impl does not bound
/// its own context on a provider trait, or that provider trait carries no `IsProviderFor` marker),
/// so a plain supertrait such as `Send` is skipped rather than mis-walked.
fn consumer_can_use_obligation<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
    consumer: ty::PolyTraitPredicate<'tcx>,
) -> Option<ty::PolyTraitPredicate<'tcx>> {
    let can_use_did = find_cgp_trait(tcx, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;
    let consumer_ref = consumer.skip_binder().trait_ref;
    let provider_did = consumer_provider_trait(tcx, consumer_ref.def_id)?;
    let marker = provider_marker(tcx, provider_did)?;

    // The component's extra parameters are grouped into the `Params` slot exactly as CGP groups
    // them: none as the unit `()`, a single one bare, several as a tuple.
    let extra: Vec<Ty<'tcx>> = consumer_ref.args.types().skip(1).collect();
    let params = match extra.as_slice() {
        [] => tcx.types.unit,
        [single] => *single,
        many => Ty::new_tup(tcx, many),
    };

    let trait_ref = ty::TraitRef::new(tcx, can_use_did, [context, marker, params]);
    Some(ty::Binder::dummy(trait_ref).upcast(tcx))
}

/// The provider trait a consumer trait pairs with, found through the consumer's blanket impl
/// `impl<C> Consumer for C where C: Provider<C>`: the `where`-bound whose self type is the impl's
/// own self is the provider trait. This is the per-consumer form of the inversion
/// [`component_map`](crate::component_map) performs across the whole trait graph.
fn consumer_provider_trait(tcx: TyCtxt<'_>, consumer_did: DefId) -> Option<DefId> {
    for &impl_did in tcx.trait_impls_of(consumer_did).blanket_impls() {
        let impl_self = tcx.type_of(impl_did).skip_binder();
        for &(clause, _) in tcx.predicates_of(impl_did).predicates {
            let Some(tp) = clause.as_trait_clause() else {
                continue;
            };
            let trait_ref = tp.skip_binder().trait_ref;
            if trait_ref.self_ty() == impl_self && is_provider_trait(tcx, trait_ref.def_id) {
                return Some(trait_ref.def_id);
            }
        }
    }
    None
}

/// The component marker a provider trait keys, read from the `IsProviderFor<Marker, …>` supertrait
/// every provider trait carries — the same second-argument marker
/// [`component_map`](crate::component_map) reads. Anchored by DefId to `cgp_component`.
fn provider_marker<'tcx>(tcx: TyCtxt<'tcx>, provider_did: DefId) -> Option<Ty<'tcx>> {
    for &(clause, _) in tcx.explicit_super_predicates_of(provider_did).skip_binder() {
        let Some(tp) = clause.as_trait_clause() else {
            continue;
        };
        let trait_ref = tp.skip_binder().trait_ref;
        if is_cgp_item(
            tcx,
            trait_ref.def_id,
            IS_PROVIDER_FOR_TRAIT,
            CGP_COMPONENT_CRATE,
        ) {
            return trait_ref.args.get(1).and_then(|arg| arg.as_type());
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
