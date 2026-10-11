//! Tests for the dependency-tree renderer. (Building and merging trees from paths is the
//! [graph](graph)'s job, tested in `graph.rs`; here only the `termtree`-backed rendering.)

use cargo_cgp_error_processing::tree::{DependencyTree, render_dependency_tree};

/// A single spine — the common cascade shape — renders as a `cargo tree`-style chain with no
/// trailing newline.
#[test]
fn renders_linear_spine() {
    let tree = DependencyTree::node(
        "consumer trait impl `CanCalculateArea` for context `Rectangle`",
        vec![DependencyTree::node(
            "provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`",
            vec![DependencyTree::node(
                "trait impl `HasRectangleFields` for `Rectangle`",
                vec![DependencyTree::leaf(
                    "field trait impl `HasField` with field `height` for `Rectangle`",
                )],
            )],
        )],
    );

    let rendered = render_dependency_tree(&tree);

    assert_eq!(
        rendered,
        "\
consumer trait impl `CanCalculateArea` for context `Rectangle`
└─ provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
  └─ trait impl `HasRectangleFields` for `Rectangle`
    └─ field trait impl `HasField` with field `height` for `Rectangle`"
    );
    assert!(!rendered.ends_with('\n'), "no trailing newline for a note");
}

/// A branching node renders each child with the box-drawing connectors.
#[test]
fn renders_branches() {
    let tree = DependencyTree::node(
        "consumer trait impl `CanHandle` for context `App`",
        vec![
            DependencyTree::leaf("field trait impl `HasField` with field `name` for `App`"),
            DependencyTree::leaf("field trait impl `HasField` with field `age` for `App`"),
        ],
    );

    assert_eq!(
        render_dependency_tree(&tree),
        "\
consumer trait impl `CanHandle` for context `App`
├─ field trait impl `HasField` with field `name` for `App`
└─ field trait impl `HasField` with field `age` for `App`"
    );
}
