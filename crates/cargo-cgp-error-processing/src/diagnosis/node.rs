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
use crate::diagnosis::wording::{dependency_tree_leaf, quoted_list};

/// One interior hop of a dependency chain — a wiring step the walk descended — tagged by the
/// `CGP-E1xx` rendering template it takes. The trait-bearing variants keep the trait reference *with*
/// its generic arguments (`CanCalculateArea<f64>`), rendered in full — every CGP construct a chain
/// names is shown as written, since the type a reader is tracing is the point of the chain. Rendering
/// is a
/// rustc-free concern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DepNode {
    /// `CGP-E101` — a hop through a consumer-trait impl. `is_context` says whether the type the
    /// trait is implemented for carries wiring of its own, which is what makes it a context; any
    /// other type, such as a type argument the consumer is required of, is named plainly.
    Consumer {
        trait_ref: String,
        context: String,
        is_context: bool,
    },
    /// `CGP-E102` — a hop through a provider's provider-trait impl.
    Provider {
        trait_ref: String,
        context: String,
        provider: String,
    },
    /// `CGP-E103` — a hop through a namespace/`open` `RedirectLookup<Table, Path>`, labeled with
    /// the `Table` the path is looked up in: the context for its own redirects, or an aggregate
    /// provider for one that opens its own components. `key` is the dispatched value (`<Outer>`);
    /// it is part of the node's *identity* — so two lookups along the same route for different keys
    /// stay distinct nodes rather than merging in the graph — but not its rendered label, since
    /// that dispatched value already shows on the child provider node.
    Redirect {
        path: String,
        table: String,
        key: String,
    },
    /// `CGP-E104` — a hop through any other trait (a user's blanket trait, a wrapper, or an ordinary
    /// bound restated as an impl).
    Trait { trait_ref: String, self_ty: String },
    /// `CGP-E104` — a hop through a trait implemented for a partial value of extensible data, a
    /// builder or an extractor, named in words since its type is generated.
    PartialTrait {
        trait_ref: String,
        value: PartialValue,
    },
}

/// A partial value an extensible-data operation runs on (a record builder or an enum extractor),
/// described by what it builds or matches and the state of each field or variant. Its type is one a
/// derive generates (`__PartialPerson<IsPresent, IsNothing>`), whose markers record each field's
/// state, so a chain names it in words instead: ``the builder of `Person` with `first_name` set``.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartialValue {
    /// Whether it builds a record or matches an enum, and how it holds an enum's payloads.
    pub kind: PartialKind,
    /// The struct or enum it builds or matches, e.g. `Person`.
    pub target: String,
    /// The fields a builder holds, or the variants an extractor has ruled out, in declaration order.
    pub marked: Vec<String>,
    /// The fields a builder holds as optional (the optional builder's state), in declaration order.
    pub optional: Vec<String>,
}

/// What kind of partial value a [`PartialValue`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartialKind {
    /// A record builder, from `builder()` or `into_builder()`.
    Builder,
    /// An enum extractor that owns its payloads, from `to_extractor()`.
    Extractor,
    /// An enum extractor over shared references, from `extractor_ref()`.
    RefExtractor,
    /// An enum extractor over mutable references, from `extractor_mut()`.
    MutExtractor,
}

impl PartialValue {
    /// The value in words: ``the builder of `Person` with `first_name` set``, or ``the extractor of
    /// `Shape` with `Circle` ruled out``.
    pub fn describe(&self) -> String {
        let target = &self.target;
        match self.kind {
            PartialKind::Builder => {
                let mut states = Vec::new();
                if !self.marked.is_empty() {
                    states.push(format!("{} set", quoted_list(&self.marked)));
                }
                if !self.optional.is_empty() {
                    states.push(format!("{} optional", quoted_list(&self.optional)));
                }
                if states.is_empty() {
                    format!("the builder of `{target}` with no field set")
                } else {
                    format!("the builder of `{target}` with {}", states.join(" and "))
                }
            }
            PartialKind::Extractor | PartialKind::RefExtractor | PartialKind::MutExtractor => {
                let noun = match self.kind {
                    PartialKind::RefExtractor => "reference extractor",
                    PartialKind::MutExtractor => "mutable reference extractor",
                    _ => "extractor",
                };
                if self.marked.is_empty() {
                    format!("the {noun} of `{target}`")
                } else {
                    format!(
                        "the {noun} of `{target}` with {} ruled out",
                        quoted_list(&self.marked)
                    )
                }
            }
        }
    }
}

impl DepNode {
    /// The rendered label — identical to the template this node replaced in `diagnosis::labels`.
    pub fn render(&self) -> String {
        match self {
            DepNode::Consumer {
                trait_ref,
                context,
                is_context,
            } => {
                let subject = if *is_context { "context " } else { "" };
                format!(
                    "[{DEP_CONSUMER_TRAIT_IMPL}] consumer trait impl `{trait_ref}` for {subject}`{context}`"
                )
            }
            DepNode::Provider {
                trait_ref,
                context,
                provider,
            } => format!(
                "[{DEP_PROVIDER_TRAIT_IMPL}] provider trait impl `{trait_ref}` with context `{context}` for provider `{provider}`"
            ),
            DepNode::Redirect { path, table, .. } => {
                format!("[{DEP_REDIRECT_LOOKUP}] redirect lookup to `{path}` in `{table}`")
            }
            DepNode::Trait { trait_ref, self_ty } => {
                format!("[{DEP_TRAIT_IMPL}] trait impl `{trait_ref}` for `{self_ty}`")
            }
            DepNode::PartialTrait { trait_ref, value } => {
                format!(
                    "[{DEP_TRAIT_IMPL}] trait impl `{trait_ref}` for {}",
                    value.describe()
                )
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
