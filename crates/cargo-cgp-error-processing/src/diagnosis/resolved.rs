//! The recovered root cause(s) of a check failure, in owned rustc-free form.
//!
//! The driver's typed resolver builds these from live compiler state and hands them to the
//! [wording](super::wording) and [plan](super::plan) here; keeping the model rustc-free is what
//! lets the whole diagnosis-to-text layer be unit-tested without a `TyCtxt`.

use std::ops::Deref;

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::node::{ChainNode, DepNode};

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

/// The root causes of one failure: **one [`Cause`] per distinct leaf**, each holding every path that
/// reaches it.
///
/// That invariant is the whole point of the type. The wording, the de-duplication signature, the
/// coalescing, and the derive `help`s all read a cause list expecting each leaf once, and a list
/// holding one leaf twice does not fail loudly — it makes a downstream reader count one mistake
/// several times, which surfaced as `` the fields `name`, `name`, and `name` `` in a lead. It used to
/// be re-established by hand with a `merge_causes_by_leaf` call at each of the five places a
/// `Resolved` is built, so a sixth would have inherited the bug silently. Here there is no way to
/// construct a `Causes` that violates it: every constructor normalizes, and the field is private.
///
/// Reads go through [`Deref`] to `[Cause]`, so consumers keep slice ergonomics while construction
/// stays controlled.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Causes(Vec<Cause>);

impl Causes {
    /// Collect `(leaf, path)` sub-chains — one per way the walk reached a root cause — grouping the
    /// paths that reach one leaf onto a single cause. This is what the walk's own result folds
    /// through, and what keeps a shared capability's diamond intact: a leaf reached twice keeps both
    /// routes rather than only the first.
    pub fn from_sub_chains(chains: impl IntoIterator<Item = (Leaf, Vec<ChainNode>)>) -> Self {
        let mut causes = Causes::default();
        for (leaf, path) in chains {
            causes.add(leaf, path);
        }
        causes
    }

    /// Merge several failures' causes into one set — the emitter's coalesced block, and any anchor
    /// that collects a walk per wired component. Total and associative, where the old
    /// concat-then-remember-to-merge was neither.
    pub fn union(parts: impl IntoIterator<Item = Self>) -> Self {
        let mut merged = Causes::default();
        for part in parts {
            for cause in part.0 {
                merged.add_cause(cause);
            }
        }
        merged
    }

    /// Head every path with `node`, so a recovered CGP chain hangs beneath the trait the programmer
    /// actually wrote (the wrapper an `impl` block names). Prepending a single constant hop cannot
    /// change any leaf's identity, so the grouping is unaffected and the operation needs no re-merge.
    pub fn headed_by(&self, node: &DepNode) -> Self {
        Causes(
            self.0
                .iter()
                .map(|cause| Cause {
                    leaf: cause.leaf.clone(),
                    paths: cause
                        .paths
                        .iter()
                        .map(|path| {
                            let mut path = path.clone();
                            path.insert(0, ChainNode::Hop(node.clone()));
                            path
                        })
                        .collect(),
                })
                .collect(),
        )
    }

    /// File one `(leaf, path)` sub-chain under its leaf.
    fn add(&mut self, leaf: Leaf, path: Vec<ChainNode>) {
        self.add_cause(Cause {
            leaf,
            paths: vec![path],
        });
    }

    /// Fold `cause` in, extending the entry for its leaf rather than adding a second one — the single
    /// place the invariant is maintained, which is why every constructor routes through here. An exact
    /// duplicate path is dropped: the [dependency graph](crate::DependencyGraph) merges paths by
    /// structural identity when it renders, so keeping it would change nothing but the work.
    fn add_cause(&mut self, cause: Cause) {
        match self.0.iter_mut().find(|seen| seen.leaf == cause.leaf) {
            Some(existing) => {
                for path in cause.paths {
                    if !existing.paths.contains(&path) {
                        existing.paths.push(path);
                    }
                }
            }
            None => self.0.push(cause),
        }
    }
}

impl FromIterator<Cause> for Causes {
    /// Collect ready-made causes, folding any that name one leaf together — so collecting cannot
    /// produce a `Causes` that breaks the invariant either.
    fn from_iter<I: IntoIterator<Item = Cause>>(iter: I) -> Self {
        let mut causes = Causes::default();
        for cause in iter {
            causes.add_cause(cause);
        }
        causes
    }
}

impl Deref for Causes {
    type Target = [Cause];

    fn deref(&self) -> &Self::Target {
        &self.0
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
    /// The root causes, one entry per distinct leaf, in first-seen order.
    pub causes: Causes,
}
