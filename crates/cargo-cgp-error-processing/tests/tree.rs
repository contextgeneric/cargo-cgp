//! Tests for the dependency-tree renderer.

use cargo_cgp_error_processing::tree::{
    DependencyTree, merge_dependency_forest, render_dependency_tree,
};

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

/// A root-first chain of labels folds into a single-spine tree; an empty chain is `None`.
#[test]
fn from_chain_folds_labels_into_a_spine() {
    let tree =
        DependencyTree::from_chain(vec!["root".to_owned(), "mid".to_owned(), "leaf".to_owned()])
            .expect("a non-empty chain folds");
    assert_eq!(
        tree,
        DependencyTree::node(
            "root",
            vec![DependencyTree::node(
                "mid",
                vec![DependencyTree::leaf("leaf")]
            )],
        )
    );
    assert_eq!(DependencyTree::from_chain(Vec::new()), None);
}

/// A short two-node spine for the merge tests: `root` over the given leaf.
fn spine(root: &str, mid: &str, leaf: &str) -> DependencyTree {
    DependencyTree::node(
        root,
        vec![DependencyTree::node(mid, vec![DependencyTree::leaf(leaf)])],
    )
}

/// Two chains that share a prefix merge into one tree: the shared ancestors appear once and the
/// divergence branches, so the merged tree ends at both distinct leaves.
#[test]
fn merges_chains_sharing_a_prefix() {
    let a = spine(
        "root `App`",
        "getter `HasName` for `App`",
        "missing field `name`",
    );
    let b = spine(
        "root `App`",
        "getter `HasAge` for `App`",
        "missing field `age`",
    );

    let merged = merge_dependency_forest(&[a, b]);
    assert_eq!(merged.len(), 1, "a shared root yields a single merged tree");

    assert_eq!(
        render_dependency_tree(&merged[0]),
        "\
root `App`
├─ getter `HasName` for `App`
│ └─ missing field `name`
└─ getter `HasAge` for `App`
  └─ missing field `age`"
    );
}

/// Chains that share every node but the leaf merge down to the last shared node, which then carries
/// both leaves as children.
#[test]
fn merges_chains_sharing_all_but_the_leaf() {
    let a = spine(
        "root `R`",
        "getter `HasFields` for `R`",
        "missing field `height`",
    );
    let b = spine(
        "root `R`",
        "getter `HasFields` for `R`",
        "missing field `width`",
    );

    let merged = merge_dependency_forest(&[a, b]);
    assert_eq!(
        render_dependency_tree(&merged[0]),
        "\
root `R`
└─ getter `HasFields` for `R`
  ├─ missing field `height`
  └─ missing field `width`"
    );
}

/// Chains with distinct roots share no ancestor, so they stay separate roots in the forest — the
/// caller keeps them as separate notes rather than forcing a shared parent.
#[test]
fn keeps_chains_with_distinct_roots_separate() {
    let a = spine("root `A`", "getter `G1` for `A`", "missing field `x`");
    let b = spine("root `B`", "getter `G2` for `B`", "missing field `y`");

    let merged = merge_dependency_forest(&[a, b]);
    assert_eq!(merged.len(), 2, "distinct roots are not merged");
}
