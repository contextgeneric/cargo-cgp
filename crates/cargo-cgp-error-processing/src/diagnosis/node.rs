//! Structured dependency-graph nodes and their rendering.
//!
//! A dependency chain is a sequence of these nodes rather than pre-rendered strings, so the
//! [graph](super::graph) can compare nodes for identity (merging a hop or leaf several paths reach
//! in common). Each [`DepNode`] variant is one
//! `CGP-E1xx` chain-hop class carrying the names that class needs; the terminal root cause is the
//! existing [`Leaf`], and a [`ChainNode`] is one or the other. Rendering reproduces exactly the
//! label templates these replaced.

use crate::code::{
    DEP_CONSUMER_TRAIT_IMPL, DEP_PROVIDER_TRAIT_IMPL, DEP_REDIRECT_LOOKUP, DEP_TRAIT_IMPL,
};
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::wording::dependency_tree_leaf;

/// One interior hop of a dependency chain — a wiring step the walk descended — tagged by the
/// `CGP-E1xx` rendering template it takes. The trait-bearing variants keep the trait reference *with*
/// its generic arguments (`CanCalculateArea<f64>`), rendered in full — every CGP construct a chain
/// names is shown as written, since the type a reader is tracing is the point of the chain. Rendering
/// is a
/// rustc-free concern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DepNode {
    /// `CGP-E101` — a hop through the context's own consumer-trait impl.
    Consumer { trait_ref: String, context: String },
    /// `CGP-E102` — a hop through a provider's provider-trait impl.
    Provider {
        trait_ref: String,
        context: String,
        provider: String,
    },
    /// `CGP-E104` — a hop through a namespace/`open` `RedirectLookup`. `key` is the dispatched value
    /// (`<Outer>`); it is part of the node's *identity* — so two lookups along the same route for
    /// different keys stay distinct nodes rather than merging in the graph — but not its rendered
    /// label, since that dispatched value already shows on the child provider node.
    Redirect {
        path: String,
        context: String,
        key: String,
    },
    /// `CGP-E105` — a hop through any other trait (a user capability, a wrapper, or an ordinary
    /// bound restated as an impl).
    Trait { trait_ref: String, self_ty: String },
}

impl DepNode {
    /// The rendered label — identical to the template this node replaced in `diagnosis::labels`.
    pub fn render(&self) -> String {
        match self {
            DepNode::Consumer { trait_ref, context } => format!(
                "[{DEP_CONSUMER_TRAIT_IMPL}] consumer trait impl `{trait_ref}` for context `{context}`"
            ),
            DepNode::Provider {
                trait_ref,
                context,
                provider,
            } => format!(
                "[{DEP_PROVIDER_TRAIT_IMPL}] provider trait impl `{trait_ref}` with context `{context}` for provider `{provider}`"
            ),
            DepNode::Redirect { path, context, .. } => {
                format!("[{DEP_REDIRECT_LOOKUP}] redirect lookup to `{path}` in `{context}`")
            }
            DepNode::Trait { trait_ref, self_ty } => {
                format!("[{DEP_TRAIT_IMPL}] trait impl `{trait_ref}` for `{self_ty}`")
            }
        }
    }
}

/// A node of the rendered dependency graph — an interior [`DepNode`] hop or the terminal root-cause
/// [`Leaf`]. Node identity (used by the graph to merge shared nodes) is structural equality on this
/// enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChainNode {
    Hop(DepNode),
    Leaf(Leaf),
}

impl ChainNode {
    /// The rendered label of this node — the hop template, or the terminal leaf's tree form.
    pub fn render(&self) -> String {
        match self {
            ChainNode::Hop(hop) => hop.render(),
            ChainNode::Leaf(leaf) => dependency_tree_leaf(leaf),
        }
    }
}
