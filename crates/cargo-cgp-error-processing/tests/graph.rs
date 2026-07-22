//! Tests for the dependency graph: building a DAG from paths and rendering it with shared-node
//! `(*)` dedup, over hand-built structured nodes and with no compiler in the loop. Each rendered
//! tree is pinned as an `insta` inline snapshot.

use cargo_cgp_error_processing::{ChainNode, DepNode, DependencyGraph, Leaf};

/// An interior `trait impl` hop labeled `name` (self type fixed to `Ctx`).
fn hop(name: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Trait {
        trait_ref: name.to_owned(),
        self_ty: "Ctx".to_owned(),
    })
}

/// A terminal leaf rendered as `the trait bound \`name\` is not satisfied`.
fn leaf(name: &str) -> ChainNode {
    ChainNode::Leaf(Leaf::Bound {
        summary: name.to_owned(),
    })
}

/// A redirect-lookup hop along `route` dispatching `key` (identity carries the key; render does not).
fn redirect(route: &str, key: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Redirect {
        path: route.to_owned(),
        context: "App".to_owned(),
        key: key.to_owned(),
    })
}

fn render(paths: &[Vec<ChainNode>]) -> String {
    DependencyGraph::from_paths(paths).render()
}

#[test]
fn renders_a_linear_spine() {
    insta::assert_snapshot!(render(&[vec![hop("A"), hop("B"), leaf("D")]]), @r"
    [CGP-E105] trait impl `A` for `Ctx`
    └─ [CGP-E105] trait impl `B` for `Ctx`
      └─ the trait bound `D` is not satisfied
    ");
}

#[test]
fn merges_a_shared_prefix_into_one_branching_tree() {
    // Two paths sharing `Con → Prov` diverge into two leaves — the shape a provider with two unmet
    // dependencies produces. The shared prefix appears once and the tree branches beneath it.
    let out = render(&[
        vec![hop("Con"), hop("Prov"), hop("HasFirst"), leaf("first")],
        vec![hop("Con"), hop("Prov"), hop("HasLast"), leaf("last")],
    ]);
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `Con` for `Ctx`
    └─ [CGP-E105] trait impl `Prov` for `Ctx`
      ├─ [CGP-E105] trait impl `HasFirst` for `Ctx`
      │ └─ the trait bound `first` is not satisfied
      └─ [CGP-E105] trait impl `HasLast` for `Ctx`
        └─ the trait bound `last` is not satisfied
    ");
}

#[test]
fn a_subsuming_chain_drops_the_contained_root() {
    // `Density → Area → D` and `Area → D`: `Area` is a head of the second path but also a child in
    // the first, so it is not a top-level root. Only the deeper chain renders, with `Area` nested.
    let out = render(&[
        vec![hop("Density"), hop("Area"), leaf("D")],
        vec![hop("Area"), leaf("D")],
    ]);
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `Density` for `Ctx`
    └─ [CGP-E105] trait impl `Area` for `Ctx`
      └─ the trait bound `D` is not satisfied
    ");
}

#[test]
fn independent_roots_converging_on_one_leaf_both_render() {
    // Two unrelated chains reach the same leaf `D`. Neither subsumes the other, so both are roots;
    // the shared leaf hides no subtree, so it is drawn in full under each (no `(*)`).
    let out = render(&[vec![hop("A"), leaf("D")], vec![hop("B"), leaf("D")]]);
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `A` for `Ctx`
    └─ the trait bound `D` is not satisfied
    [CGP-E105] trait impl `B` for `Ctx`
    └─ the trait bound `D` is not satisfied
    ");
}

#[test]
fn a_diamond_shows_the_shared_subtree_once() {
    // Two top-level nodes `A` and `B` both depend on `C`, which depends on the missing `D`. `C` is
    // one node with two parents: expanded under `A` (reached first), referenced with `(*)` under `B`,
    // so the root cause `D` is shown exactly once.
    let out = render(&[
        vec![hop("A"), hop("C"), leaf("D")],
        vec![hop("B"), hop("C"), leaf("D")],
    ]);
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `A` for `Ctx`
    └─ [CGP-E105] trait impl `C` for `Ctx`
      └─ the trait bound `D` is not satisfied
    [CGP-E105] trait impl `B` for `Ctx`
    └─ [CGP-E105] trait impl `C` for `Ctx` (*)
    ");
}

#[test]
fn a_super_root_over_two_branches_shares_the_subtree() {
    // `S` depends on both `A` and `B`, which both depend on `C → D`. `S` is the only head that is
    // not a descendant, so it is the single root; `C` is expanded once and referenced with `(*)`.
    let out = render(&[
        vec![hop("S"), hop("A"), hop("C"), leaf("D")],
        vec![hop("S"), hop("B"), hop("C"), leaf("D")],
    ]);
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `S` for `Ctx`
    ├─ [CGP-E105] trait impl `A` for `Ctx`
    │ └─ [CGP-E105] trait impl `C` for `Ctx`
    │   └─ the trait bound `D` is not satisfied
    └─ [CGP-E105] trait impl `B` for `Ctx`
      └─ [CGP-E105] trait impl `C` for `Ctx` (*)
    ");
}

#[test]
fn a_repeated_trait_elides_its_generics_against_its_parent() {
    // Two distinct provider nodes sharing a trait reference (`Handler<Big>`) form a parent/child
    // pair; the child's generics elide to `<…>` because it repeats its parent's trait. (They are
    // distinct nodes — different providers — so the graph does not merge them.)
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
    insta::assert_snapshot!(out, @r"
    [CGP-E102] provider trait impl `Handler<Big>` with context `Ctx` for provider `P1`
    └─ [CGP-E102] provider trait impl `Handler<…>` with context `Ctx` for provider `P2`
      └─ the trait bound `D` is not satisfied
    ");
}

#[test]
fn a_hop_whose_generics_differ_from_its_parent_keeps_its_full_form() {
    // The complement of the elision test: two provider nodes on the *same* trait whose generic
    // arguments differ (`ValueEncoder<Outer>` then `ValueEncoder<Vec<Mid>>`) each keep their full
    // parameters — elision fires only on an *exact* repeat, never on a changing generic.
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
    assert!(!out.contains("<…>"), "a changing generic is not elided");
    insta::assert_snapshot!(out, @r"
    [CGP-E102] provider trait impl `ValueEncoder<Outer>` with context `Ctx` for provider `EncodeRecord`
    └─ [CGP-E102] provider trait impl `ValueEncoder<Vec<Mid>>` with context `Ctx` for provider `EncodeIterator`
      └─ the trait bound `D` is not satisfied
    ");
}

#[test]
fn a_within_path_label_repeat_stays_a_linear_chain() {
    // A single path passes through a same-labelled hop `R` twice — a recursive descent (a redirect
    // resolving `Outer` then `Inner`) whose label omits the key. It is one path, so the repeat must
    // stay a distinct node: a linear chain, not a false cycle that would `(*)`-fold onto itself.
    let out = render(&[vec![hop("R"), hop("X"), hop("R"), leaf("D")]]);
    assert!(!out.contains("(*)"), "a within-path repeat is not a cycle");
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `R` for `Ctx`
    └─ [CGP-E105] trait impl `X` for `Ctx`
      └─ [CGP-E105] trait impl `R` for `Ctx`
        └─ the trait bound `D` is not satisfied
    ");
}

#[test]
fn cross_path_redirects_differing_only_by_key_stay_distinct() {
    // Two lookups along the same route (`@VBC`) for different keys render the *same* label but are
    // different nodes (the key is part of their identity). In separate paths they must not merge
    // into a false diamond — each leads to its own leaf, with no `(*)`.
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
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `Outer` for `Ctx`
    └─ [CGP-E104] redirect lookup to `@VBC` in `App`
      └─ the trait bound `outer-missing` is not satisfied
    [CGP-E105] trait impl `Inner` for `Ctx`
    └─ [CGP-E104] redirect lookup to `@VBC` in `App`
      └─ the trait bound `inner-missing` is not satisfied
    ");
}

#[test]
fn cross_path_redirects_with_the_same_key_merge() {
    // Same route *and* same key is genuinely the same lookup, so it merges across paths like any
    // shared node: expanded under the first, `(*)`-referenced under the second.
    let out = render(&[
        vec![hop("A"), redirect("@VBC", "Key"), leaf("missing")],
        vec![hop("B"), redirect("@VBC", "Key"), leaf("missing")],
    ]);
    insta::assert_snapshot!(out, @r"
    [CGP-E105] trait impl `A` for `Ctx`
    └─ [CGP-E104] redirect lookup to `@VBC` in `App`
      └─ the trait bound `missing` is not satisfied
    [CGP-E105] trait impl `B` for `Ctx`
    └─ [CGP-E104] redirect lookup to `@VBC` in `App` (*)
    ");
}

#[test]
fn an_empty_path_set_renders_empty() {
    assert!(DependencyGraph::from_paths(&[]).is_empty());
    insta::assert_snapshot!(render(&[]), @"");
}

#[test]
fn a_cycle_across_paths_terminates_and_is_marked() {
    // Two paths that between them form a cycle — `A → B` and `B → A` — leave every head also a
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
