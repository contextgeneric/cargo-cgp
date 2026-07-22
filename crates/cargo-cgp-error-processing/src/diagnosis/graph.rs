//! The dependency graph: a DAG assembled from root→leaf paths, rendered `cargo tree`-style.
//!
//! The resolver produces one path of [`ChainNode`]s per way a root cause is reached. This module
//! folds a set of such paths into a directed acyclic graph — merging every node several paths reach
//! in common by structural identity — and renders it. A node reached again is drawn once and
//! referenced with `(*)` (the convention `cargo tree` uses), so a shared dependency, a diamond, a
//! super-root, and independent chains converging on one leaf all render correctly. The whole thing
//! is a pure function over the structured nodes, so every shape is unit-tested without a compiler.

use std::collections::HashMap;

use crate::diagnosis::node::ChainNode;
use crate::tree::{DependencyTree, render_dependency_tree};

/// A dependency DAG built from root→leaf paths. Nodes are deduplicated by structural identity, so a
/// hop or leaf several paths share is one node with several parents and/or children; edges and path
/// heads are kept in first-seen order for deterministic rendering.
pub struct DependencyGraph {
    /// Every distinct node, indexed by id (insertion order).
    nodes: Vec<ChainNode>,
    /// Each node's dependency children, unique and in first-seen order.
    children: Vec<Vec<usize>>,
    /// Whether each node is ever some node's child — the complement of "is a top-level root".
    is_child: Vec<bool>,
    /// The head (first node) of each input path, unique and in first-seen order.
    heads: Vec<usize>,
}

impl DependencyGraph {
    /// Fold a set of root→leaf paths into the graph. A node equal to one already seen in *another*
    /// path reuses its id, so a dependency several paths share becomes one node (a diamond); each
    /// adjacent pair adds a child edge, and each path's first node is recorded as a head.
    ///
    /// A label that repeats *within a single path* is kept a distinct node, not merged. A linear
    /// descent can pass through two hops that render identically yet mean different things — a
    /// recursive `RedirectLookup` resolving `Outer` then `Inner`, say, whose label omits the key —
    /// and merging those would fold the spine into a false cycle. Only a genuine cross-path
    /// convergence merges.
    pub fn from_paths(paths: &[Vec<ChainNode>]) -> Self {
        let mut index: HashMap<ChainNode, usize> = HashMap::new();
        let mut nodes: Vec<ChainNode> = Vec::new();
        let mut children: Vec<Vec<usize>> = Vec::new();
        let mut is_child: Vec<bool> = Vec::new();
        let mut heads: Vec<usize> = Vec::new();

        for path in paths {
            // The ids already placed on *this* path, so a within-path label repeat is not merged
            // back onto an ancestor of itself.
            let mut used: Vec<usize> = Vec::new();
            let mut parent: Option<usize> = None;
            for node in path {
                let id = match index.get(node) {
                    Some(&existing) if !used.contains(&existing) => existing,
                    _ => {
                        nodes.push(node.clone());
                        children.push(Vec::new());
                        is_child.push(false);
                        let id = nodes.len() - 1;
                        // Register only the first occurrence of a label, so a later path still finds
                        // the canonical node; a within-path repeat stays unregistered and distinct.
                        index.entry(node.clone()).or_insert(id);
                        id
                    }
                };
                used.push(id);
                match parent {
                    None => {
                        if !heads.contains(&id) {
                            heads.push(id);
                        }
                    }
                    Some(parent) => {
                        if !children[parent].contains(&id) {
                            children[parent].push(id);
                        }
                        is_child[id] = true;
                    }
                }
                parent = Some(id);
            }
        }

        DependencyGraph {
            nodes,
            children,
            is_child,
            heads,
        }
    }

    /// Whether the graph holds no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The top-level roots: path heads that are not also some node's child. A head that appears
    /// inside another path (one consumer's chain running through another) is therefore not rendered
    /// as a second root — which is how subsumption falls out. Falls back to every head if that set is
    /// empty (a pathological all-cyclic input), so rendering never yields nothing.
    fn roots(&self) -> Vec<usize> {
        let roots: Vec<usize> = self
            .heads
            .iter()
            .copied()
            .filter(|&head| !self.is_child[head])
            .collect();
        if roots.is_empty() {
            self.heads.clone()
        } else {
            roots
        }
    }

    /// Render the graph as one `cargo tree`-style diagram per root, joined by newlines and with no
    /// trailing newline (so a caller can drop it into a diagnostic note).
    pub fn render(&self) -> String {
        let mut expanded = vec![false; self.nodes.len()];
        self.roots()
            .into_iter()
            .map(|root| render_dependency_tree(&self.expand(root, None, &mut expanded)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Expand node `id` into a render tree. Its generics are elided when its trait exactly repeats
    /// its parent's (`parent_trait`, the parent's *own* un-elided trait). A node reached a second
    /// time whose subtree was already drawn is emitted as a `(*)` reference rather than re-expanded;
    /// the `expanded` marks double as cycle protection, since each node is expanded at most once.
    fn expand(
        &self,
        id: usize,
        parent_trait: Option<&str>,
        expanded: &mut [bool],
    ) -> DependencyTree {
        let node = &self.nodes[id];
        let own_trait = node.elidable_trait().map(str::to_owned);
        let label = match (&own_trait, parent_trait) {
            (Some(this), Some(parent))
                if this == parent && this.ends_with('>') && this.contains('<') =>
            {
                node.with_generics_elided().render()
            }
            _ => node.render(),
        };

        let has_children = !self.children[id].is_empty();
        if expanded[id] && has_children {
            return DependencyTree::leaf(format!("{label} (*)"));
        }
        expanded[id] = true;

        let kids = self.children[id]
            .clone()
            .into_iter()
            .map(|child| self.expand(child, own_trait.as_deref(), expanded))
            .collect();
        DependencyTree::node(label, kids)
    }
}
