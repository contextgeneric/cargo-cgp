//! The recovered root cause(s) of a check failure, in owned rustc-free form.
//!
//! The driver's typed resolver builds these from live compiler state and hands them to the
//! [wording](super::wording) and [plan](super::plan) here; keeping the model rustc-free is what
//! lets the whole diagnosis-to-text layer be unit-tested without a `TyCtxt`.

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::node::ChainNode;

/// One recovered root cause: the leaf it names for the note heading, and every root→leaf path that
/// reaches it. A leaf reached one way has a single path — the common case; a leaf reached through a
/// shared capability several providers depend on has several, which the
/// [dependency graph](crate::DependencyGraph) merges when it renders. Keeping several paths on one
/// cause preserves the *one cause per distinct leaf* invariant the de-duplication signatures, the
/// consumer coalescing, and the derive `help`s all rely on. For a coalesced underived-field cause the
/// heading `leaf` is the merged [`Leaf::UnderivedFields`], while the paths still terminate at the
/// individual per-field leaves so the graph branches to each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cause {
    /// The leaf the note heading names (its lead, code, and derive `help` come from this).
    pub leaf: Leaf,
    /// Every root→leaf path that reaches this cause, each a full chain of nodes (interior hops then
    /// the terminal leaf node).
    pub paths: Vec<Vec<ChainNode>>,
}

impl Cause {
    /// A stable key that de-duplicates a leaf reached by several dependency paths — see
    /// [`Leaf::key`].
    pub fn key(&self) -> &str {
        self.leaf.key()
    }
}

/// The recovered root cause(s) of a check failure, in owned form so they outlive the inference
/// contexts they were read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The type the failing trait(s) are (not) implemented for — the header subject. Usually the
    /// checked context itself (`Rectangle`), but for a wrapper implemented on a *foreign* type that
    /// merely holds the context (`Router<Arc<MockApp>>`), it is that foreign type, and
    /// [`subject_is_context`](Resolved::subject_is_context) is then `false`.
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
    /// Whether [`context`](Resolved::context) is the checked CGP context itself (`true`, the usual
    /// case) or a foreign wrapper type the failing trait is implemented for that merely holds the
    /// context (`false`) — as `Router<Arc<MockApp>>` does for a routing trait. It selects whether the
    /// header calls the subject a `context` or names it plainly, so a foreign wrapper is not
    /// mislabelled a context.
    pub subject_is_context: bool,
    /// One entry per distinct root cause, in first-seen order.
    pub causes: Vec<Cause>,
}
