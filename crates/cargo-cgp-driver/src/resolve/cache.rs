//! A per-compilation memoization cache over the typed resolver's walk.
//!
//! See `docs/implementation/cached-dependency-resolution.md`. CGP wiring is lazy, so one mistake
//! surfaces the same failure at many sites — the `check_components!` entry, every hand-written
//! `impl` that references the broken consumer, and each call — and a shared capability is a diamond
//! reached from several parents. This cache memoizes the walk **at every node**, storing each node's
//! owned, rustc-free sub-result so a repeated obligation is resolved once and reused.
//!
//! The key is a [`StableHash`] fingerprint of the region-erased obligation together with its root
//! context — the context is part of the input because the walk's rendering compares node self-types
//! against it (see the design document). Only the fingerprint feeds `Hash`/`Eq`; the rendered
//! `obligation`/`context` strings are carried purely so the store can be inspected. The fingerprint
//! is `Copy` and lifetime-free, and the value is owned, so entries persist past any `TyCtxt`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use cargo_cgp_error_processing::Leaf;
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::fx::FxHashSet;
use rustc_data_structures::stable_hash::{StableHash, StableHasher};
use rustc_middle::ty::{self, Ty, TyCtxt};

/// One root-cause sub-chain of a node's subtree: the classified leaf and the label chain from this
/// node down to it (node-rooted — the node's own label first, the leaf's label last), so a parent
/// reuses it by prepending its own label and the root turns it into a `Cause`.
#[derive(Clone)]
pub(crate) struct SubCause {
    pub(crate) leaf: Leaf,
    pub(crate) labels: Vec<String>,
}

/// The owned result of resolving one node's subtree: its sub-causes, the fingerprints of every
/// obligation reachable within the subtree (for the reuse disjointness check), and whether any guard
/// (the cycle guard or the depth cap) curtailed it — an incomplete subtree is never cached.
#[derive(Clone)]
pub(crate) struct SubResult {
    pub(crate) causes: Vec<SubCause>,
    pub(crate) reachable: FxHashSet<Fingerprint>,
    pub(crate) incomplete: bool,
}

impl SubResult {
    /// A branch the cycle guard or depth cap cut: no causes, and flagged incomplete so it is never
    /// cached and taints every ancestor up the stack.
    pub(crate) fn cut() -> Self {
        Self {
            causes: Vec::new(),
            reachable: FxHashSet::default(),
            incomplete: true,
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
        }
    }
}

/// A `StableHash` fingerprint of an already region-erased obligation *alone* (no context), used for
/// the subtree reachable sets and the ancestor-disjointness check — the context is constant within a
/// walk, so it need not enter these fingerprints.
pub(crate) fn pred_fingerprint<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
) -> Fingerprint {
    tcx.with_stable_hashing_context(|mut hcx| {
        let mut hasher = StableHasher::new();
        pred.stable_hash(&mut hcx, &mut hasher);
        hasher.finish()
    })
}

/// Identity of a resolved node. Only [`fingerprint`](Self::fingerprint) feeds `Hash`/`Eq`; the
/// rendered fields are debug-only, carried so the store can be dumped, and never affect a lookup —
/// so they need not be injective.
#[derive(Clone, Debug)]
pub(crate) struct NodeKey {
    /// The sole basis for `Hash`/`Eq`: a `StableHash` fingerprint of the region-erased obligation
    /// and its root context.
    fingerprint: Fingerprint,
    /// Debug-only: the obligation and context rendered as text, for inspecting the store through the
    /// derived `Debug`. Read only by `Debug`, which dead-code analysis does not count.
    #[allow(dead_code)]
    obligation: String,
    #[allow(dead_code)]
    context: String,
}

impl NodeKey {
    /// Build a key from an **already region-erased** obligation and its root context. The
    /// fingerprint is faithful because `StableHash` encodes each `DefId` by its stable path
    /// identity, so two same-named types from different modules never collide.
    pub(crate) fn new<'tcx>(
        tcx: TyCtxt<'tcx>,
        obligation: ty::PolyTraitPredicate<'tcx>,
        context: Ty<'tcx>,
    ) -> Self {
        let fingerprint = tcx.with_stable_hashing_context(|mut hcx| {
            let mut hasher = StableHasher::new();
            obligation.stable_hash(&mut hcx, &mut hasher);
            context.stable_hash(&mut hcx, &mut hasher);
            hasher.finish()
        });
        Self {
            fingerprint,
            obligation: obligation.to_string(),
            context: context.to_string(),
        }
    }
}

impl PartialEq for NodeKey {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

impl Eq for NodeKey {}

impl Hash for NodeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fingerprint.hash(state);
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
    /// inserted — see the walk.
    pub(crate) fn insert(&self, key: NodeKey, value: SubResult) {
        self.entries.borrow_mut().insert(key, value);
    }
}
