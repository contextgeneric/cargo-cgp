//! Tests for the dependency-tree renderer.

use cargo_cgp_error_processing::tree::{DependencyTree, render_dependency_tree};

/// A single spine — the common cascade shape — renders as a `cargo tree`-style chain with no
/// trailing newline.
#[test]
fn renders_linear_spine() {
    let tree = DependencyTree::node(
        "`Rectangle` uses `CanCalculateArea` (provider `RectangleArea`)",
        vec![DependencyTree::node(
            "requires `HasRectangleFields`",
            vec![DependencyTree::leaf("requires field `height` (missing)")],
        )],
    );

    let rendered = render_dependency_tree(&tree);

    assert_eq!(
        rendered,
        "\
`Rectangle` uses `CanCalculateArea` (provider `RectangleArea`)
└── requires `HasRectangleFields`
    └── requires field `height` (missing)"
    );
    assert!(!rendered.ends_with('\n'), "no trailing newline for a note");
}

/// A branching node renders each child with the box-drawing connectors.
#[test]
fn renders_branches() {
    let tree = DependencyTree::node(
        "`App` uses `CanHandle`",
        vec![
            DependencyTree::leaf("requires field `name` (missing)"),
            DependencyTree::leaf("requires field `age` (missing)"),
        ],
    );

    assert_eq!(
        render_dependency_tree(&tree),
        "\
`App` uses `CanHandle`
├── requires field `name` (missing)
└── requires field `age` (missing)"
    );
}
