//! Tests for the dependency graph: building a DAG from paths and rendering it with shared-node
//! `(*)` dedup, over hand-built structured nodes and with no compiler in the loop. Each rendered
//! tree is pinned as an `insta` inline snapshot.

use std::collections::HashSet;

use cargo_cgp_error_processing::{ChainNode, DepNode, DependencyGraph, Leaf};

use super::support::{self, trait_hop};

/// An interior `trait impl` hop labeled `name` (self type fixed to `Ctx`).
fn hop(name: &str) -> ChainNode {
    trait_hop(name, "Ctx")
}

/// A terminal ordinary-bound leaf, rendered as
/// `[CGP-E117] the trait bound \`name\` is not satisfied`.
fn leaf(name: &str) -> ChainNode {
    ChainNode::Leaf(Leaf::Bound {
        summary: name.to_owned(),
    })
}

/// A redirect-lookup hop along `route` dispatching `key` (identity carries the key; render does
/// not).
fn redirect(route: &str, key: &str) -> ChainNode {
    support::redirect(route, "App", key)
}

fn render(paths: &[Vec<ChainNode>]) -> String {
    DependencyGraph::from_paths(paths).render()
}

// Shows: a single path renders as one root with each node nested one level under the one before.
#[test]
fn renders_a_linear_spine() {
    insta::assert_snapshot!(render(&[vec![hop("A"), hop("B"), leaf("D")]]), @"
    [CGP-E104] trait impl `A` for `Ctx`
    └─ [CGP-E104] trait impl `B` for `Ctx`
      └─ [CGP-E117] the trait bound `D` is not satisfied
    ");
}

// Shows: a prefix several paths share renders once, branching where the paths diverge
//   (`DependencyGraph::from_paths`'s doc).
#[test]
fn merges_a_shared_prefix_into_one_branching_tree() {
    // Two paths sharing `Con → Prov` diverge into two leaves: the shape a provider with two unmet
    // dependencies produces. The shared prefix appears once and the tree branches beneath it.
    let out = render(&[
        vec![hop("Con"), hop("Prov"), hop("HasFirst"), leaf("first")],
        vec![hop("Con"), hop("Prov"), hop("HasLast"), leaf("last")],
    ]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `Con` for `Ctx`
    └─ [CGP-E104] trait impl `Prov` for `Ctx`
      ├─ [CGP-E104] trait impl `HasFirst` for `Ctx`
      │ └─ [CGP-E117] the trait bound `first` is not satisfied
      └─ [CGP-E104] trait impl `HasLast` for `Ctx`
        └─ [CGP-E117] the trait bound `last` is not satisfied
    ");
}

// Shows: a path head that is another path's child is not a top-level root, so only the deeper chain
//   renders (`DependencyGraph::roots`'s doc).
#[test]
fn a_subsuming_chain_drops_the_contained_root() {
    // `Density → Area → D` and `Area → D`: `Area` is a head of the second path but also a child in
    // the first, so it is not a top-level root. Only the deeper chain renders, with `Area` nested.
    let out = render(&[
        vec![hop("Density"), hop("Area"), leaf("D")],
        vec![hop("Area"), leaf("D")],
    ]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `Density` for `Ctx`
    └─ [CGP-E104] trait impl `Area` for `Ctx`
      └─ [CGP-E117] the trait bound `D` is not satisfied
    ");
}

// Shows: roots that meet only at a leaf both render, and the leaf is drawn in full under each,
//   since a leaf hides no subtree (`expand`'s doc).
#[test]
fn independent_roots_converging_on_one_leaf_both_render() {
    // Two unrelated chains reach the same leaf `D`. Neither subsumes the other, so both are roots;
    // the shared leaf hides no subtree, so it is drawn in full under each (no `(*)`).
    let out = render(&[vec![hop("A"), leaf("D")], vec![hop("B"), leaf("D")]]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `A` for `Ctx`
    └─ [CGP-E117] the trait bound `D` is not satisfied
    [CGP-E104] trait impl `B` for `Ctx`
    └─ [CGP-E117] the trait bound `D` is not satisfied
    ");
}

// Shows: a node with two parents is expanded under the first and referenced as `(*)` under the
//   second, so its subtree appears once (`expand`'s doc).
#[test]
fn a_diamond_shows_the_shared_subtree_once() {
    // Two top-level nodes `A` and `B` both depend on `C`, which depends on the missing `D`. `C` is
    // one node with two parents: expanded under `A` (reached first), referenced with `(*)` under
    // `B`, so the root cause `D` is shown exactly once.
    let out = render(&[
        vec![hop("A"), hop("C"), leaf("D")],
        vec![hop("B"), hop("C"), leaf("D")],
    ]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `A` for `Ctx`
    └─ [CGP-E104] trait impl `C` for `Ctx`
      └─ [CGP-E117] the trait bound `D` is not satisfied
    [CGP-E104] trait impl `B` for `Ctx`
    └─ [CGP-E104] trait impl `C` for `Ctx` (*)
    ");
}

// Shows: the single head that is no node's child is the only root, and a subtree both its branches
//   share is expanded once (`roots` and `expand`'s docs).
#[test]
fn a_super_root_over_two_branches_shares_the_subtree() {
    // `S` depends on both `A` and `B`, which both depend on `C → D`. `S` is the only head that is
    // not a descendant, so it is the single root; `C` is expanded once and referenced with `(*)`.
    let out = render(&[
        vec![hop("S"), hop("A"), hop("C"), leaf("D")],
        vec![hop("S"), hop("B"), hop("C"), leaf("D")],
    ]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `S` for `Ctx`
    ├─ [CGP-E104] trait impl `A` for `Ctx`
    │ └─ [CGP-E104] trait impl `C` for `Ctx`
    │   └─ [CGP-E117] the trait bound `D` is not satisfied
    └─ [CGP-E104] trait impl `B` for `Ctx`
      └─ [CGP-E104] trait impl `C` for `Ctx` (*)
    ");
}

// Shows: a hop that repeats its parent's trait still names every generic argument, and distinct
//   nodes on one trait stay distinct (`expand`'s doc: every CGP construct is rendered in full).
#[test]
fn a_repeated_trait_still_names_its_generics_in_full() {
    // Two distinct provider nodes sharing a trait reference (`Handler<Big>`) form a parent/child
    // pair. A dispatch chain restates a program-sized `Code` at every step, but shortening it would
    // hide the type the reader is tracing, so every CGP construct a chain names renders as written.
    // They are distinct nodes, with different providers, so the graph does not merge them.
    let parent = ChainNode::Hop(DepNode::Provider {
        trait_ref: "Handler<Big>".to_owned(),
        context: "Ctx".to_owned(),
        provider: "P1".to_owned(),
    });
    let child = ChainNode::Hop(DepNode::Provider {
        trait_ref: "Handler<Big>".to_owned(),
        context: "Ctx".to_owned(),
        provider: "P2".to_owned(),
    });
    let out = render(&[vec![parent, child, leaf("D")]]);
    assert!(!out.contains("<…>"), "no CGP construct is ever elided");
    insta::assert_snapshot!(out, @"
    [CGP-E102] provider trait impl `Handler<Big>` with context `Ctx` for provider `P1`
    └─ [CGP-E102] provider trait impl `Handler<Big>` with context `Ctx` for provider `P2`
      └─ [CGP-E117] the trait bound `D` is not satisfied
    ");
}

// Shows: hops on one trait with different arguments each render in full (`expand`'s doc).
#[test]
fn a_hop_whose_generics_differ_from_its_parent_keeps_its_full_form() {
    // The companion of the test above: two provider nodes on the *same* trait whose generic
    // arguments differ (`ValueEncoder<Outer>` then `ValueEncoder<Vec<Mid>>`). Both are rendered in
    // full, as every hop is.
    let parent = ChainNode::Hop(DepNode::Provider {
        trait_ref: "ValueEncoder<Outer>".to_owned(),
        context: "Ctx".to_owned(),
        provider: "EncodeRecord".to_owned(),
    });
    let child = ChainNode::Hop(DepNode::Provider {
        trait_ref: "ValueEncoder<Vec<Mid>>".to_owned(),
        context: "Ctx".to_owned(),
        provider: "EncodeIterator".to_owned(),
    });
    let out = render(&[vec![parent, child, leaf("D")]]);
    assert!(!out.contains("<…>"), "no CGP construct is ever elided");
    insta::assert_snapshot!(out, @"
    [CGP-E102] provider trait impl `ValueEncoder<Outer>` with context `Ctx` for provider `EncodeRecord`
    └─ [CGP-E102] provider trait impl `ValueEncoder<Vec<Mid>>` with context `Ctx` for provider `EncodeIterator`
      └─ [CGP-E117] the trait bound `D` is not satisfied
    ");
}

// Shows: a label repeated within one path stays a distinct node, so the chain stays linear with no
//   `(*)` (`from_paths`'s doc).
#[test]
fn a_within_path_label_repeat_stays_a_linear_chain() {
    // A single path passes through a same-labelled hop `R` twice: a recursive descent (a redirect
    // resolving `Outer` then `Inner`) whose label omits the key. It is one path, so the repeat must
    // stay a distinct node: a linear chain, not a false cycle that would `(*)`-fold onto itself.
    let out = render(&[vec![hop("R"), hop("X"), hop("R"), leaf("D")]]);
    assert!(!out.contains("(*)"), "a within-path repeat is not a cycle");
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `R` for `Ctx`
    └─ [CGP-E104] trait impl `X` for `Ctx`
      └─ [CGP-E104] trait impl `R` for `Ctx`
        └─ [CGP-E117] the trait bound `D` is not satisfied
    ");
}

// Shows: redirect lookups along one route for different keys stay distinct nodes across paths,
//   since the key is part of their identity, so neither is `(*)`.
#[test]
fn cross_path_redirects_differing_only_by_key_stay_distinct() {
    // Two lookups along the same route (`@VBC`) for different keys render the *same* label but are
    // different nodes (the key is part of their identity). In separate paths they must not merge
    // into a false diamond: each leads to its own leaf, with no `(*)`.
    let out = render(&[
        vec![
            hop("Outer"),
            redirect("@VBC", "OuterKey"),
            leaf("outer-missing"),
        ],
        vec![
            hop("Inner"),
            redirect("@VBC", "InnerKey"),
            leaf("inner-missing"),
        ],
    ]);
    assert!(
        !out.contains("(*)"),
        "distinct-key redirects are not a shared node"
    );
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `Outer` for `Ctx`
    └─ [CGP-E103] redirect lookup to `@VBC` in `App`
      └─ [CGP-E117] the trait bound `outer-missing` is not satisfied
    [CGP-E104] trait impl `Inner` for `Ctx`
    └─ [CGP-E103] redirect lookup to `@VBC` in `App`
      └─ [CGP-E117] the trait bound `inner-missing` is not satisfied
    ");
}

// Shows: a redirect lookup with the same route and key in two paths merges into one node,
//   `(*)`-referenced under the second (`from_paths`'s doc).
#[test]
fn cross_path_redirects_with_the_same_key_merge() {
    // Same route *and* same key is genuinely the same lookup, so it merges across paths like any
    // shared node: expanded under the first, `(*)`-referenced under the second.
    let out = render(&[
        vec![hop("A"), redirect("@VBC", "Key"), leaf("missing")],
        vec![hop("B"), redirect("@VBC", "Key"), leaf("missing")],
    ]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `A` for `Ctx`
    └─ [CGP-E103] redirect lookup to `@VBC` in `App`
      └─ [CGP-E117] the trait bound `missing` is not satisfied
    [CGP-E104] trait impl `B` for `Ctx`
    └─ [CGP-E103] redirect lookup to `@VBC` in `App` (*)
    ");
}

// Property: an empty path set builds an empty graph, which renders as an empty string.
#[test]
fn an_empty_path_set_renders_empty() {
    assert!(DependencyGraph::from_paths(&[]).is_empty());
    insta::assert_snapshot!(render(&[]), @"");
}

// Property: for all path sets whose heads are all children (a cycle), rendering terminates and
//   marks the re-reached node `(*)` (`roots` and `expand`'s docs).
#[test]
fn a_cycle_across_paths_terminates_and_is_marked() {
    // Two paths that between them form a cycle (`A → B` and `B → A`) leave every head also a
    // child, so `roots()` falls back to all heads. The `expanded` set caps each node at one
    // expansion, so rendering terminates (rather than looping) and marks the re-reached node `(*)`.
    // This is a pathological input the resolver's cycle guard should never emit; the test pins the
    // defensive termination guarantee the module promises regardless.
    let out = render(&[vec![hop("A"), hop("B")], vec![hop("B"), hop("A")]]);
    assert!(out.contains('A') && out.contains('B'), "both nodes appear");
    assert!(
        out.contains("(*)"),
        "the re-reached node is marked, not re-expanded"
    );
}

/// Two graphs rendered against one shared `seen` set: the shape a compilation's separate error
/// blocks form when one mistake surfaces in several diagnostics that do not de-duplicate. The
/// second keeps its own prefix and truncates at the subtree the first already drew.
// Shows: a graph rendered against an earlier graph's `seen` set keeps its own hops, references the
//   shared subtree as `(*)`, and still ends at the root cause (`render_seen` and `leaves_below`'s
//   docs).
#[test]
fn a_second_graph_elides_what_the_first_drew() {
    let mut seen = HashSet::new();

    let first = DependencyGraph::from_paths(&[vec![hop("Check"), hop("Shared"), leaf("missing")]]);
    insta::assert_snapshot!(first.render_seen(&mut seen), @"
    [CGP-E104] trait impl `Check` for `Ctx`
    └─ [CGP-E104] trait impl `Shared` for `Ctx`
      └─ [CGP-E117] the trait bound `missing` is not satisfied
    ");

    // `Wrapper` is this graph's own hop and renders; `Shared` was drawn above, so its intervening
    // hops are referenced rather than repeated, but the chain still bottoms out at the root cause,
    // which the reader of *this* block would otherwise never see.
    let second = DependencyGraph::from_paths(&[vec![
        hop("Wrapper"),
        hop("Shared"),
        hop("Deep"),
        leaf("missing"),
    ]]);
    assert!(!second.fully_elided_by(&seen), "its own root is unseen");
    insta::assert_snapshot!(second.render_seen(&mut seen), @"
    [CGP-E104] trait impl `Wrapper` for `Ctx`
    └─ [CGP-E104] trait impl `Shared` for `Ctx` (*)
      └─ [CGP-E117] the trait bound `missing` is not satisfied
    ");
}

/// A branch elided *within* one render needs no such terminator: the subtree it points at, root
/// cause included, is right above it in the same note. Only a cross-render elision re-states the
/// cause, since there the expansion lives in another block.
// Shows: a `(*)` reference within one render needs no restated leaf, since the expansion is above
//   it in the same note.
#[test]
fn a_within_render_reference_stays_bare() {
    let out = render(&[
        vec![hop("Top"), hop("Left"), hop("Shared"), leaf("missing")],
        vec![hop("Top"), hop("Right"), hop("Shared"), leaf("missing")],
    ]);
    insta::assert_snapshot!(out, @"
    [CGP-E104] trait impl `Top` for `Ctx`
    ├─ [CGP-E104] trait impl `Left` for `Ctx`
    │ └─ [CGP-E104] trait impl `Shared` for `Ctx`
    │   └─ [CGP-E117] the trait bound `missing` is not satisfied
    └─ [CGP-E104] trait impl `Right` for `Ctx`
      └─ [CGP-E104] trait impl `Shared` for `Ctx` (*)
    ");
}

/// A graph whose *every* root was already drawn has nothing of its own to show, so a caller is told
/// to drop the chain rather than head a lone `(*)` with a promise of one.
// Property: for all graphs whose every root is in `seen`, `fully_elided_by(seen)` holds, and for
//   none drawn yet it does not (`fully_elided_by`'s doc).
#[test]
fn a_wholly_redundant_graph_reports_itself_fully_elided() {
    let mut seen = HashSet::new();
    let paths = vec![vec![hop("Shared"), leaf("missing")]];

    let first = DependencyGraph::from_paths(&paths);
    assert!(!first.fully_elided_by(&seen), "nothing drawn yet");
    let _ = first.render_seen(&mut seen);

    assert!(DependencyGraph::from_paths(&paths).fully_elided_by(&seen));
}

/// A bare leaf hides no subtree, so it is drawn in full wherever a chain bottoms out on it: a
/// graph that is only a leaf is never reported as fully elided, however often it recurs.
// Property: a graph that is only a leaf is never fully elided, however often it recurs, since a
//   leaf hides no subtree (`fully_elided_by` and `expand`'s docs).
#[test]
fn a_leaf_only_graph_is_never_elided() {
    let mut seen = HashSet::new();
    let paths = vec![vec![leaf("missing")]];
    let _ = DependencyGraph::from_paths(&paths).render_seen(&mut seen);
    assert!(!DependencyGraph::from_paths(&paths).fully_elided_by(&seen));
}
