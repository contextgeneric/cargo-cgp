//! The dependency graph: a DAG assembled from root→leaf paths, rendered `cargo tree`-style.
//!
//! The resolver produces one path of [`ChainNode`]s per way a root cause is reached. This module
//! folds a set of such paths into a directed acyclic graph — merging every node several paths reach
//! in common by structural identity — and renders it. A node reached again is drawn once and
//! referenced with `(*)` (the convention `cargo tree` uses), so a shared dependency, a diamond, a
//! super-root, and independent chains converging on one leaf all render correctly. The whole thing
//! is a pure function over the structured nodes, so every shape is unit-tested without a compiler.

use std::collections::{HashMap, HashSet};

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

    /// Whether rendering against `seen` would say nothing new — every top-level root was already
    /// drawn elsewhere, so the whole diagram would collapse to `(*)` references.
    ///
    /// A caller uses this to drop the chain entirely rather than print a
    /// `this is required through the dependency chain:` heading over a single pointer, which promises
    /// a chain and delivers none. It reports the *whole* graph being redundant, not a subtree: a
    /// partly-elided graph still has its own hops to show and renders normally.
    pub fn fully_elided_by(&self, seen: &HashSet<ChainNode>) -> bool {
        let roots = self.roots();
        !roots.is_empty()
            && roots
                .iter()
                // A childless root is a bare leaf, which is never elided, so it always renders.
                .all(|&root| !self.children[root].is_empty() && seen.contains(&self.nodes[root]))
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
        self.render_seen(&mut HashSet::new())
    }

    /// [`render`](Self::render) against a `seen` set that outlives this graph, so a node some
    /// *earlier* graph already drew is `(*)`-referenced here instead of expanded again.
    ///
    /// This is what lets a compilation's diagnostics elide across blocks. CGP wiring is lazy, so one
    /// mistake surfaces in several diagnostics that do not de-duplicate — a hand-written wrapper
    /// trait is a distinct trait from the consumer it reduces to, so it keeps its own block — and
    /// their chains can share everything below their own first few hops. Threading one `seen` through
    /// the blocks in emission order keeps each block's own prefix and truncates the shared remainder,
    /// which is the same `(*)` convention `cargo tree` uses for a subtree printed elsewhere in the
    /// output.
    ///
    /// A truncated block stays actionable on its own: its header, its fix `help`, and its
    /// `root cause:` lead all still name the cause, so only chain *detail* is elided, never what
    /// failed or how to fix it.
    pub fn render_seen(&self, seen: &mut HashSet<ChainNode>) -> String {
        let mut expanded = vec![false; self.nodes.len()];
        // Nodes this render draws are collected apart and folded into `seen` only at the end, so
        // within this render `seen` names *only* what earlier ones drew. That separation is
        // load-bearing: `seen` is keyed by node value, while a label repeating within a single path
        // is deliberately a distinct node (see [`from_paths`](Self::from_paths)), so consulting a
        // set this render is also filling would mark the second occurrence `(*)` and fold a linear
        // descent into a false cycle. Within a render, only `expanded` — keyed by id — elides.
        let mut drawn: Vec<ChainNode> = Vec::new();
        let rendered = self
            .roots()
            .into_iter()
            .map(|root| {
                render_dependency_tree(&self.expand(root, None, &mut expanded, seen, &mut drawn))
            })
            .collect::<Vec<_>>()
            .join("\n");
        seen.extend(drawn);
        rendered
    }

    /// The distinct terminal leaves reachable from `id`, in first-seen order, as render nodes. This
    /// is what an elided branch bottoms out on, so a chain whose middle was drawn in another note
    /// still ends at the root cause. Descent is bounded by a visited set, so a cyclic graph — which
    /// the resolver should never emit, but which the renderer promises to survive — terminates.
    fn leaves_below(&self, id: usize) -> Vec<DependencyTree> {
        let mut leaves = Vec::new();
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if std::mem::replace(&mut visited[current], true) {
                continue;
            }
            if matches!(self.nodes[current], ChainNode::Leaf(_)) {
                let rendered = self.nodes[current].render();
                if !leaves.contains(&rendered) {
                    leaves.push(rendered);
                }
            }
            // Reversed, so the pop order matches the children's first-seen order.
            stack.extend(self.children[current].iter().rev().copied());
        }
        leaves.into_iter().map(DependencyTree::leaf).collect()
    }

    /// Expand node `id` into a render tree. Its generics are elided when its trait exactly repeats
    /// its parent's (`parent_trait`, the parent's *own* un-elided trait). A node whose subtree was
    /// already drawn — earlier in this render (`expanded`, indexed by id) or by an earlier one
    /// (`seen`, keyed by node identity so it spans graphs) — is emitted as a `(*)` reference rather
    /// than re-expanded; the `expanded` marks double as cycle protection, since each node is expanded
    /// at most once. Only a node with children is ever elided: a leaf hides no subtree, so it is
    /// drawn in full wherever a chain bottoms out on it.
    fn expand(
        &self,
        id: usize,
        parent_trait: Option<&str>,
        expanded: &mut [bool],
        seen: &HashSet<ChainNode>,
        drawn: &mut Vec<ChainNode>,
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
        if has_children && expanded[id] {
            // Already drawn *in this render*: the subtree, root cause and all, is visible above in
            // the same note, so the marker alone points at it.
            return DependencyTree::leaf(format!("{label} (*)"));
        }
        if has_children && seen.contains(node) {
            // Drawn by an *earlier* render, in another note the reader may not have to hand. The
            // intervening hops are elided, but the chain must still bottom out at the cause it leads
            // to — a chain that stops short of the root cause is the one thing it may never do.
            return DependencyTree::node(format!("{label} (*)"), self.leaves_below(id));
        }
        expanded[id] = true;
        if has_children {
            drawn.push(node.clone());
        }

        let kids = self.children[id]
            .clone()
            .into_iter()
            .map(|child| self.expand(child, own_trait.as_deref(), expanded, seen, drawn))
            .collect();
        DependencyTree::node(label, kids)
    }
}
