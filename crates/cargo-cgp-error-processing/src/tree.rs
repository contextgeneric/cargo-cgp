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
