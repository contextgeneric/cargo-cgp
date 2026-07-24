//! The by-component use-site anchor: re-checking every component the context wires.

use cargo_cgp_error_processing::{Cause, Resolved, merge_causes_by_leaf};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::anchor::{consumer_obligation, context_candidates_from_spans};
use crate::resolve::cache::ResolveCache;
use crate::resolve::cgp_item::{
    find_cgp_trait, is_nil, is_path_cons, marker_to_consumer, path_cons_parts,
};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve the root cause(s) of a CGP wiring failure reported at a *use site* rather than a
/// `check_components!` entry — a consumer-method call (`E0599`) or any other diagnostic whose
/// obligation is not recoverable from a check impl. There is no check impl to anchor on, so the
/// context type is recovered from a diagnostic span that lands on a local struct/enum definition,
/// and every component that context wires (through its `DelegateComponent` impls) is re-checked;
/// each one that cannot be used contributes its dependency tree. `None` when no context is found
/// or no wired component fails resolvably.
pub fn resolve_use_site(tcx: TyCtxt<'_>, cache: &ResolveCache, spans: &[Span]) -> Option<Resolved> {
    // A diagnostic span can land on a provider struct as well as the real context (both are local
    // ADTs), so try each candidate and keep the first that actually wires a failing component.
    for context in context_candidates_from_spans(tcx, spans) {
        let mut causes: Vec<Cause> = Vec::new();
        let mut consumers: Vec<String> = Vec::new();
        for (marker, params) in delegated_check_targets(tcx, context) {
            // Map the wired marker to its consumer trait and walk the real obligation
            // `Ctx: Consumer<params…>`, not a `CanUseComponent`/`IsProviderFor` wrapper. `params`
            // is `()` for an ordinary (non-dispatched) component, or the recovered dispatch value
            // for an `open`-dispatched one; a component whose form holds is skipped.
            let Some((consumer_did, _)) = marker_to_consumer(tcx, marker) else {
                continue;
            };
            let Some(top) = consumer_obligation(tcx, context, consumer_did, params) else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, cache, top) {
                for consumer in resolved.consumers {
                    if !consumers.contains(&consumer) {
                        consumers.push(consumer);
                    }
                }
                // Merged by leaf below, so a cause several wired components reach keeps every
                // component's path rather than only the first's.
                causes.extend(resolved.causes);
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
                causes: merge_causes_by_leaf(&causes),
            });
        }
    }
    None
}

/// The `(marker, params)` pairs a use-site failure re-checks — each mapped to its real consumer
/// obligation `Ctx: Consumer<params…>` — read from the context's `DelegateComponent<Key>` impls. A
/// `DelegateComponent` key is one of three shapes, and each yields a different re-check:
///
/// - A **bare component marker** (`ItemEncoderComponent`) re-checks with the unit parameter `()`,
///   the parameterless form an ordinary component's use-site failure exercises — *unless* the same
///   component is `open`-dispatched (below), in which case its `()` form is meaningless (there is no
///   unit-keyed value) and would report a spurious `@Component.()` redirect, so it is skipped.
/// - An **`open`-dispatch redirect path** (`PathCons<ItemEncoderComponent, PathCons<Value, Nil>>`,
///   emitted by an `@Component.Value:` entry) is *not* a component marker — re-checking it as one
///   reports the internal `PathCons` spine as a bogus consumer trait. Instead the real dispatch
///   parameter is recovered from the path, re-checking `CanUseComponent<Component, Value>` so the
///   failure is traced with the value the context actually wired (a longer, non-two-segment path is
///   skipped rather than mis-rendered).
/// - A **blanket-forwarding key** — a bare type parameter (`__Key__`) — is the impl a `namespace …;`
///   join emits (`impl<__Key__> DelegateComponent<__Key__> for Ctx`), which forwards *every* lookup
///   to the namespace rather than naming a concrete component. It is not a real wired key, and
///   re-checking a free parameter bottoms out on `__Key__: Sized` noise under a bogus `__Key__`
///   consumer-trait header, so it is skipped (as the generic-catch-all `open` value is). The
///   context's concrete wiring lives in the namespace, out of this per-context view, so a
///   pure namespace join yields no target and the use-site resolver declines rather than fabricate a
///   cause.
fn delegated_check_targets<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
) -> Vec<(Ty<'tcx>, Ty<'tcx>)> {
    let Some(delegate_did) = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
    else {
        return Vec::new();
    };
    let context = tcx.erase_and_anonymize_regions(context);

    let keys: Vec<Ty<'tcx>> = tcx
        .all_impls(delegate_did)
        .filter(|&impl_did| {
            let impl_self = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
            tcx.erase_and_anonymize_regions(impl_self) == context
        })
        // `DelegateComponent<Key>` — args are `[Self, Key]`.
        .map(|impl_did| {
            let key = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip()
                .args
                .type_at(1);
            tcx.erase_and_anonymize_regions(key)
        })
        .collect();

    // The components reached through an `open`-dispatch redirect, so a bare marker for one of them
    // is not also re-checked with the spurious `()` parameter.
    let dispatched: Vec<Ty<'tcx>> = keys
        .iter()
        .filter_map(|&key| open_dispatch_target(tcx, key).map(|(comp, _)| comp))
        .collect();

    let mut targets = Vec::new();
    for &key in &keys {
        if let Some((comp, value)) = open_dispatch_target(tcx, key) {
            // A generic catch-all open entry (`<'a, T> &'a T: SerializeDeref`) keeps a free type
            // parameter in its recovered value; re-checking `CanUseComponent<Comp, &T>` bottoms out
            // on `T: Sized` noise rather than a real gap, and every concrete value the entry serves
            // is re-checked through its own entry, so skip it.
            if !value.has_param() {
                targets.push((comp, value));
            }
        } else if !is_path_cons(tcx, key) && !dispatched.contains(&key) && !key.has_param() {
            // A bare marker with no free parameter is a concrete wired component. A key that *is*
            // (or contains) a free parameter is the `namespace …;` blanket forwarding (`__Key__`),
            // not a real component, so it is dropped rather than re-checked into `__Key__: Sized`
            // noise.
            targets.push((key, tcx.types.unit));
        }
    }
    targets
}

/// Recover the `(component, value)` an `open`-dispatch redirect key stands for — the two-segment
/// path an `@Component.Value:` wiring entry emits — so a use-site re-check can use the real dispatch
/// value rather than the raw path. The key is `PathCons<Component, PathCons<Value, Tail>>`, where
/// `Tail` is the `Nil` terminator or the generic wildcard the macro leaves for prefix matching; both
/// mark a two-segment key. `None` when `key` is not such a path — a bare marker, or a genuine
/// three-plus-segment namespace route (whose `Tail` is a further `PathCons`), which the caller skips
/// rather than mis-render.
fn open_dispatch_target<'tcx>(tcx: TyCtxt<'tcx>, key: Ty<'tcx>) -> Option<(Ty<'tcx>, Ty<'tcx>)> {
    let comp_rest = path_cons_parts(tcx, key)?;
    let value_rest = path_cons_parts(tcx, comp_rest.1)?;
    if !is_path_terminator(tcx, value_rest.1) {
        return None;
    }
    Some((comp_rest.0, value_rest.0))
}

/// Whether `ty` ends a `PathCons` spine at the second segment: either CGP's `Nil` terminator or the
/// generic wildcard parameter the `open` expansion leaves as the tail (so the entry prefix-matches).
/// A further `PathCons` here means the path has a third segment, so it is not a two-segment key.
fn is_path_terminator(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    is_nil(tcx, ty) || matches!(ty.kind(), ty::Param(_))
}
