//! Building the sub-result of a node that ends a path: a terminal leaf, a cause, or a projection mismatch.

use cargo_cgp_error_processing::{ChainNode, Leaf};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, Upcast as _};

use crate::resolve::cache::{SubCause, SubResult, pred_fingerprint};
use crate::resolve::classify::{classify_leaf, is_reportable_leaf};
use crate::resolve::label::label_for;
use crate::resolve::query::ProjectionMismatch;

/// Build the sub-result for a terminal leaf `leaf_ref` (a `HasField`, an impl-less bound, or a
/// foreign bound with nothing to follow). Drops a leaf still carrying a call-site placeholder (an
/// unknowable `_: Send`) and a non-reportable plumbing dead-end, in both cases as a complete
/// no-cause result. Not cached — the classification reads `parent`, which lies outside the leaf.
pub fn terminal_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    context: Ty<'tcx>,
    self_fp: Fingerprint,
) -> SubResult {
    // A leaf still carrying one of the seed's unknown-parameter placeholders (the call-site anchor's
    // stand-in for a call's inferred input) is a bound on a type the recovery could not know, so it
    // is dropped rather than reported as a fabricated `_: Send`.
    if erased.has_placeholders() {
        return SubResult::empty(self_fp);
    }
    // A path that bottoms out on pure wiring plumbing (a routing dead-end) is not a root cause.
    if !is_reportable_leaf(tcx, leaf_ref, context, parent) {
        return SubResult::empty(self_fp);
    }
    let leaf = classify_leaf(tcx, leaf_ref, context, parent, None);
    let mut path = Vec::new();
    // A seed that is itself the cause (a data operation with no impl at all, such as a finalize on
    // a builder with a field unset) heads the chain with its own step, so the chain names the
    // operation the cause is about rather than restating the cause alone.
    if parent.is_none()
        && let Some(hop) = label_for(tcx, erased, context)
    {
        path.push(ChainNode::Hop(hop));
    }
    path.push(ChainNode::Leaf(leaf.clone()));
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    SubResult {
        causes: vec![SubCause { path, leaf }],
        reachable,
        incomplete: false,
        cut: None,
    }
}

/// Build the sub-result for a cause the node itself states, already classified as `leaf`: the node
/// is a chain hop (its label, when it has one) and `leaf` ends the path.
pub fn cause_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    context: Ty<'tcx>,
    leaf: Leaf,
    self_fp: Fingerprint,
) -> SubResult {
    let mut path = Vec::new();
    if let Some(node_hop) = label_for(tcx, erased, context) {
        path.push(ChainNode::Hop(node_hop));
    }
    path.push(ChainNode::Leaf(leaf.clone()));
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    SubResult {
        causes: vec![SubCause { leaf, path }],
        reachable,
        incomplete: false,
        cut: None,
    }
}

/// Build the sub-result for a projection mismatch: the impl matched with every trait-clause holding,
/// but an associated type it projects through carries the wrong type. The node itself becomes a chain
/// hop (its label) and the projection's own trait ref is the terminal leaf, carrying the mismatch.
/// `parent_ref` (the node's own trait) is that leaf's parent.
pub fn projection_result<'tcx>(
    tcx: TyCtxt<'tcx>,
    erased: ty::PolyTraitClause<'tcx>,
    parent_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    mismatch: ProjectionMismatch<'tcx>,
    self_fp: Fingerprint,
) -> SubResult {
    let leaf_ref = mismatch.trait_ref;
    let leaf_poly: ty::PolyTraitClause<'tcx> = ty::Binder::dummy(leaf_ref).upcast(tcx);
    if leaf_poly.has_placeholders() || leaf_poly.references_error() {
        return SubResult::empty(self_fp);
    }
    let parent = Some(parent_ref);
    if !is_reportable_leaf(tcx, leaf_ref, context, parent) {
        return SubResult::empty(self_fp);
    }
    let leaf = classify_leaf(tcx, leaf_ref, context, parent, Some(mismatch));
    let mut path = Vec::new();
    if let Some(node_hop) = label_for(tcx, erased, context) {
        path.push(ChainNode::Hop(node_hop));
    }
    path.push(ChainNode::Leaf(leaf.clone()));
    let mut reachable = FxHashSet::default();
    reachable.insert(self_fp);
    reachable.insert(pred_fingerprint(tcx, leaf_poly));
    SubResult {
        causes: vec![SubCause { leaf, path }],
        reachable,
        incomplete: false,
        cut: None,
    }
}
