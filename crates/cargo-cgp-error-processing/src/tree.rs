//! Rendering a resolved check failure's dependency chain as a `cargo tree`-style tree.
//!
//! When the driver's typed resolver traces a check failure down to its root cause, it recovers
//! the whole transitive chain that led there — the checked capability, each provider and
//! capability it depends on, and the missing leaf. This module turns that chain, handed over as
//! a compiler-free [`DependencyTree`], into the indented text that goes in the replacement
//! diagnostic's one dependency note.
//!
//! It lives in this rustc-free crate, like [`rewrite`](crate::rewrite), so the rendering is
//! unit-tested on any toolchain even though the *data* is built in the driver from typed
//! compiler state. The box-drawing is delegated to [`termtree`], the same lightweight renderer
//! `cargo tree` and other cargo tools use.

use termtree::{GlyphPalette, Tree};

/// Compact box-drawing glyphs: two columns of indentation per depth level instead of `termtree`'s
/// default four. The connector is `└─ ` in place of `└── ` (one dash instead of two), and the
/// continuation under it is a single space instead of three, so each level nests only two columns.
/// CGP dependency chains nest deeply — a realistic wiring bottoms out tens of levels down — so
/// halving the per-level indent keeps a deep tree from marching off the right margin. The item
/// indent (`─ `, two columns) is wider than the skip indent (` `, one column); `termtree` only
/// requires the two to match for a *multiline* node label, which these single-line labels never are.
const COMPACT_GLYPHS: GlyphPalette = GlyphPalette {
    middle_item: "├",
    last_item: "└",
    item_indent: "─ ",
    middle_skip: "│",
    last_skip: " ",
    skip_indent: " ",
};

/// One node of a resolved check failure's dependency chain: a human-readable `label` and its
/// dependencies as `children`. The root is the checked capability, each descent is a further
/// dependency, and the deepest node is the missing root cause. A linear cascade is a single
/// spine; a provider with several unmet dependencies branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyTree {
    /// The already-formatted description of this dependency step.
    pub label: String,
    /// The dependencies of this step, rendered indented beneath it.
    pub children: Vec<DependencyTree>,
}

impl DependencyTree {
    /// A leaf node — a dependency step with nothing beneath it.
    pub fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }

    /// A node with the given children.
    pub fn node(label: impl Into<String>, children: Vec<DependencyTree>) -> Self {
        Self {
            label: label.into(),
            children,
        }
    }
}

/// Render a dependency tree as `cargo tree`-style indented text, e.g.
///
/// ```text
/// consumer trait impl `CanCalculateArea` for context `Rectangle`
/// └─ provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
///   └─ trait impl `HasRectangleFields` for `Rectangle`
///     └─ field trait impl `HasField` with field `height` for `Rectangle`
/// ```
///
/// The returned string has no trailing newline, so a caller can drop it straight into a
/// diagnostic note.
pub fn render_dependency_tree(tree: &DependencyTree) -> String {
    to_termtree(tree)
        .with_glyphs(COMPACT_GLYPHS)
        .to_string()
        .trim_end()
        .to_owned()
}

/// Convert the compiler-free [`DependencyTree`] into a [`termtree::Tree`] for rendering.
fn to_termtree(node: &DependencyTree) -> Tree<String> {
    let mut rendered = Tree::new(node.label.clone());
    for child in &node.children {
        rendered.push(to_termtree(child));
    }
    rendered
}

/// Merge a forest of dependency trees into one, sharing every node that several trees reach by the
/// *same* path of labels: a common ancestor is shown once, and the point where the trees diverge
/// branches beneath it. This collapses the several root→leaf chains of a multi-root-cause failure —
/// each a linear spine that repeats the whole shared prefix — into a single tree whose branches end
/// at their distinct leaves.
///
/// Sibling nodes are keyed by their (already-rendered) `label`, and equal-labelled siblings are
/// fused and their children merged in turn; first-seen order is preserved. Trees whose *roots* carry
/// different labels stay separate roots in the returned forest — nothing is forced under a shared
/// parent that the inputs do not actually share — so a caller merges only when the result is a
/// single tree (a genuine common ancestor) and otherwise keeps the chains apart.
pub fn merge_dependency_forest(trees: &[DependencyTree]) -> Vec<DependencyTree> {
    merge_siblings(&trees.iter().collect::<Vec<_>>())
}

/// Merge a set of sibling nodes: group by label (first-seen order), and for each group emit one node
/// whose children are the recursively-merged children of every node in the group.
fn merge_siblings(nodes: &[&DependencyTree]) -> Vec<DependencyTree> {
    let mut groups: Vec<(&str, Vec<&DependencyTree>)> = Vec::new();
    for &node in nodes {
        match groups.iter_mut().find(|(label, _)| *label == node.label) {
            Some((_, group)) => group.push(node),
            None => groups.push((&node.label, vec![node])),
        }
    }
    groups
        .into_iter()
        .map(|(label, group)| {
            let children: Vec<&DependencyTree> =
                group.iter().flat_map(|node| node.children.iter()).collect();
            DependencyTree::node(label, merge_siblings(&children))
        })
        .collect()
}
