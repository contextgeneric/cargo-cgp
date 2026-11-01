//! A per-compilation memoization cache over the typed resolver's walk.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`. CGP wiring is
//! lazy, so one mistake surfaces the same failure at many sites: the `check_components!` entry,
//! every hand-written `impl` that references the broken consumer, and each call, and a shared
//! trait is a diamond reached from several parents. This cache memoizes the walk **at every
//! node**, storing each node's owned, rustc-free sub-result so a repeated obligation is resolved
//! once and reused.
//!
//! The key is a [`StableHash`] fingerprint of the region-erased obligation together with its root
//! context: the context is part of the input because the walk's rendering compares node self-types
//! against it (see the design document). The fingerprint is the key's only field, `Copy` and
//! lifetime-free, and the value is owned, so entries persist past any `TyCtxt`.

use std::cell::RefCell;
use std::collections::HashMap;

use cargo_cgp_error_processing::{ChainNode, DepNode, Leaf, WiringOverflow};
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_data_structures::stable_hash::{StableHash, StableHasher};
use rustc_middle::ty::{self, Ty, TyCtxt};

/// One root-cause sub-chain of a node's subtree: the classified leaf and the node chain from this
/// node down to it (node-rooted: this node's own hop first, the terminal leaf node last), so a
/// parent reuses it by prepending its own hop and the root groups it into a `Cause` by leaf.
#[derive(Clone)]
pub(crate) struct SubCause {
    pub(crate) leaf: Leaf,
    pub(crate) path: Vec<ChainNode>,
}

/// The owned result of resolving one node's subtree: its sub-causes, the fingerprints of every
/// obligation reachable within the subtree (for the reuse disjointness check), and whether any guard
/// (the cycle guard or the depth cap) curtailed it: an incomplete subtree is never cached.
#[derive(Clone)]
pub(crate) struct SubResult {
    pub(crate) causes: Vec<SubCause>,
    pub(crate) reachable: FxHashSet<Fingerprint>,
    pub(crate) incomplete: bool,
    /// Why a guard cut the subtree, when one did: the obligation met again (a cycle), or the depth
    /// cap reached without a repeat (a growing lookup). A wiring overflow is worded from it.
    pub(crate) cut: Option<WiringOverflow>,
}

impl SubResult {
    /// A branch the cycle guard or depth cap cut, for `reason`: no causes, and flagged incomplete so
    /// it is never cached and taints every ancestor up the stack.
    pub(crate) fn cut(reason: WiringOverflow) -> Self {
        Self {
            causes: Vec::new(),
            reachable: FxHashSet::default(),
            incomplete: true,
            cut: Some(reason),
        }
    }

    /// A complete node that reaches no reportable cause (an impl matched with nothing to report).
    pub(crate) fn empty(self_fp: Fingerprint) -> Self {
        let mut reachable = FxHashSet::default();
        reachable.insert(self_fp);
        Self {
            causes: Vec::new(),
            reachable,
            incomplete: false,
            cut: None,
        }
    }

    /// A complete node whose one cause is `leaf`, reached through `hop`, the node's own step, when
    /// it has one; `reachable` holds the fingerprints of the obligations the path covers.
    pub(crate) fn cause(hop: Option<DepNode>, leaf: Leaf, reachable: &[Fingerprint]) -> Self {
        let path = hop
            .map(ChainNode::Hop)
            .into_iter()
            .chain([ChainNode::Leaf(leaf.clone())])
            .collect();
        Self {
            causes: vec![SubCause { path, leaf }],
            reachable: reachable.iter().copied().collect(),
            incomplete: false,
            cut: None,
        }
    }
}

/// A `StableHash` fingerprint of an already region-erased obligation *alone* (no context), used for
/// the subtree reachable sets and the ancestor-disjointness check: the context is constant within a
/// walk, so it need not enter these fingerprints.
pub(crate) fn pred_fingerprint<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
) -> Fingerprint {
    tcx.with_stable_hashing_context(|mut hcx| {
        let mut hasher = StableHasher::new();
        pred.stable_hash(&mut hcx, &mut hasher);
        hasher.finish()
    })
}

/// Identity of a resolved node: a `StableHash` fingerprint of the region-erased obligation and its
/// root context.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NodeKey {
    fingerprint: Fingerprint,
}

impl NodeKey {
    /// Build a key from an **already region-erased** obligation and its root context. The
    /// fingerprint is faithful because `StableHash` encodes each `DefId` by its stable path
    /// identity, so two same-named types from different modules never collide.
    pub(crate) fn new<'tcx>(
        tcx: TyCtxt<'tcx>,
        obligation: ty::PolyTraitClause<'tcx>,
        context: Ty<'tcx>,
    ) -> Self {
        let fingerprint = tcx.with_stable_hashing_context(|mut hcx| {
            let mut hasher = StableHasher::new();
            obligation.stable_hash(&mut hcx, &mut hasher);
            context.stable_hash(&mut hcx, &mut hasher);
            hasher.finish()
        });
        Self { fingerprint }
    }
}

/// The per-compilation resolver cache. Interior mutability so it is reachable both through the
/// `&self` emitter and from inside a `ty::tls::with` closure; owned values so entries outlive the
/// inference contexts they were read from and persist for the whole compilation.
#[derive(Default)]
pub struct ResolveCache {
    entries: RefCell<HashMap<NodeKey, SubResult>>,
}

impl ResolveCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// The cached sub-result for `key`, or `None` when the node has not been resolved yet. A hit
    /// clones the stored value; the borrow is released before the clone returns, so the memo never
    /// holds it across the compute that follows a miss.
    pub(crate) fn get(&self, key: &NodeKey) -> Option<SubResult> {
        self.entries.borrow().get(key).cloned()
    }

    /// Record a complete node's sub-result. Only complete (untainted) non-terminal nodes are ever
    /// inserted; see the walk.
    pub(crate) fn insert(&self, key: NodeKey, value: SubResult) {
        self.entries.borrow_mut().insert(key, value);
    }
}
