//! The recovered root cause(s) of a check failure, in owned rustc-free form.
//!
//! The driver's typed resolver builds these from live compiler state and hands them to the
//! [wording](super::wording) and [plan](super::plan) here; keeping the model rustc-free is what
//! lets the whole diagnosis-to-text layer be unit-tested without a `TyCtxt`.

use crate::diagnosis::leaf::Leaf;
use crate::tree::DependencyTree;

/// One recovered root cause: the leaf the chain bottoms out on and the transitive dependency
/// chain that leads to it, rendered as a single spine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cause {
    /// What the chain bottoms out on.
    pub leaf: Leaf,
    /// The dependency chain from the checked component down to the leaf.
    pub tree: DependencyTree,
}

impl Cause {
    /// A stable key that de-duplicates a leaf reached by several dependency paths — the field
    /// name for a field, the bound restatement otherwise.
    pub fn key(&self) -> &str {
        self.leaf.key()
    }
}

/// The recovered root cause(s) of a check failure, in owned form so they outlive the inference
/// contexts they were read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The checked context type, e.g. `Rectangle`.
    pub context: String,
    /// The trait(s) the context fails to implement, with a generic component's extra parameters
    /// reattached (e.g. `CanCalculateArea<f64>`) — one per failing component, in first-seen order.
    /// The emitter words a rewritten main message around these.
    pub consumers: Vec<String>,
    /// Whether [`consumers`](Resolved::consumers) are CGP *consumer* traits (from a
    /// `check_components!` entry or a consumer-method call) rather than a hand-written wrapper trait
    /// implemented on the context. It selects the header wording: `the consumer trait` (`CGP-E001`)
    /// when `true`, `the trait` (`CGP-E009`) when `false` — since a wrapper such as
    /// `CanHandleApiSend` is a plain trait the programmer wrote, not a CGP consumer.
    pub consumers_are_cgp: bool,
    /// One entry per distinct root cause, in first-seen order.
    pub causes: Vec<Cause>,
}
