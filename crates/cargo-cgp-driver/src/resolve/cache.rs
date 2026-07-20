//! A per-compilation memoization cache over the typed resolver's walk.
//!
//! See `docs/implementation/cached-dependency-resolution.md`. CGP wiring is lazy, so one mistake
//! surfaces the same failure at many sites — the `check_components!` entry, every hand-written
//! `impl` that references the broken consumer, and each call — and each of those diagnostics seeds
//! the walk with the *same* obligation. This cache memoizes the walk's owned, rustc-free
//! [`Resolved`] output so that seed is resolved once and reused, rather than re-walked per site.
//!
//! The key is a [`StableHash`] fingerprint of the region-erased obligation together with its root
//! context — the context is part of the input because the walk's rendering compares node self-types
//! against it (see the design document). Only the fingerprint feeds `Hash`/`Eq`; the rendered
//! `obligation`/`context` strings are carried purely so the store can be inspected. The fingerprint
//! is `Copy` and lifetime-free, and the value is owned, so entries persist past any `TyCtxt`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use cargo_cgp_error_processing::Resolved;
use rustc_data_structures::fingerprint::Fingerprint;
use rustc_data_structures::stable_hash::{StableHash, StableHasher};
use rustc_middle::ty::{self, Ty, TyCtxt};

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
    entries: RefCell<HashMap<NodeKey, Option<Resolved>>>,
}

impl ResolveCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// The cached result for `key`, or `None` when the key has not been resolved yet. A hit clones
    /// the stored value; the borrow is released before the clone returns, so the memo never holds it
    /// across the compute that follows a miss.
    pub(crate) fn get(&self, key: &NodeKey) -> Option<Option<Resolved>> {
        self.entries.borrow().get(key).cloned()
    }

    /// Record a result — including a negative one (`None`), since a seed that declines does so
    /// deterministically and need not be re-walked.
    pub(crate) fn insert(&self, key: NodeKey, value: Option<Resolved>) {
        self.entries.borrow_mut().insert(key, value);
    }
}
