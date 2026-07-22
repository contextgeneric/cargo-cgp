//! Structured dependency-graph nodes and their rendering.
//!
//! A dependency chain is a sequence of these nodes rather than pre-rendered strings, so the
//! [graph](super::graph) can compare nodes for identity (merging a hop or leaf several paths reach
//! in common) and elide a hop's generics against its parent. Each [`DepNode`] variant is one
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
/// its generic arguments (`CanCalculateArea<f64>`), since rendering and eliding them is now a
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

    /// The trait reference (with generics) this hop elides when it repeats its parent's — the
    /// consumer/provider/other-trait name — or `None` for a hop whose label carries no such trait
    /// (a `HasField` accessor or a redirect path).
    fn elidable_trait(&self) -> Option<&str> {
        match self {
            DepNode::Consumer { trait_ref, .. }
            | DepNode::Provider { trait_ref, .. }
            | DepNode::Trait { trait_ref, .. } => Some(trait_ref),
            DepNode::Redirect { .. } => None,
        }
    }

    /// A copy with the generic list of its elidable trait reduced to `<…>`. A no-op for a variant
    /// with no elidable trait or a trait reference that carries no `<…>` list.
    fn with_generics_elided(&self) -> DepNode {
        let mut node = self.clone();
        let trait_ref = match &mut node {
            DepNode::Consumer { trait_ref, .. }
            | DepNode::Provider { trait_ref, .. }
            | DepNode::Trait { trait_ref, .. } => trait_ref,
            DepNode::Redirect { .. } => return node,
        };
        if trait_ref.ends_with('>')
            && let Some(open) = trait_ref.find('<')
        {
            *trait_ref = format!("{}<…>", &trait_ref[..open]);
        }
        node
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

    /// The elidable trait reference for a hop; a leaf never elides.
    pub(crate) fn elidable_trait(&self) -> Option<&str> {
        match self {
            ChainNode::Hop(hop) => hop.elidable_trait(),
            ChainNode::Leaf(_) => None,
        }
    }

    /// This node with its generics elided when it is a hop; a leaf is returned unchanged.
    pub(crate) fn with_generics_elided(&self) -> ChainNode {
        match self {
            ChainNode::Hop(hop) => ChainNode::Hop(hop.with_generics_elided()),
            ChainNode::Leaf(_) => self.clone(),
        }
    }
}
