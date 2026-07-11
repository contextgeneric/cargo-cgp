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

use termtree::Tree;

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
/// `Rectangle` uses `CanCalculateArea` (provider `RectangleArea`)
/// └── requires `HasRectangleFields`
///     └── requires field `height` (missing)
/// ```
///
/// The returned string has no trailing newline, so a caller can drop it straight into a
/// diagnostic note.
pub fn render_dependency_tree(tree: &DependencyTree) -> String {
    to_termtree(tree).to_string().trim_end().to_owned()
}

/// Convert the compiler-free [`DependencyTree`] into a [`termtree::Tree`] for rendering.
fn to_termtree(node: &DependencyTree) -> Tree<String> {
    let mut rendered = Tree::new(node.label.clone());
    for child in &node.children {
        rendered.push(to_termtree(child));
    }
    rendered
}
