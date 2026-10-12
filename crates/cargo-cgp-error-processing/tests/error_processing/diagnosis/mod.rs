//! Tests for the rustc-free diagnosis model and its diagnostic plan.
//!
//! These exercise the wording and planning that used to live inside the driver's emitter, where
//! it could only be pinned end-to-end by a UI snapshot. Moving it into this crate makes each case
//! a plain unit test over a hand-built [`Resolved`], with no compiler in the loop. The dependency
//! chain each note carries is built by the [`DependencyGraph`] from structured node paths; the
//! graph's own rendering is pinned separately in `graph.rs`, so here the focus is the note
//! assembly (lead, heading, indentation, singular vs plural) around it.

mod headers;
mod headline;
mod helps;
mod notes;
mod plan;

use std::collections::HashMap;

use cargo_cgp_error_processing::diagnosis::{assoc_mismatch_leaf, mismatch_leaf, quoted_list};
use cargo_cgp_error_processing::rewrite::{ComponentNameMap, ComponentTraitNames};
use cargo_cgp_error_processing::{
    Cause, ChainNode, DepNode, DependencyGraph, DiagKind, FieldIssue, Leaf, PendingNote, Resolved,
    cause_headline, cause_note, cause_notes, cause_signature, coalesced_headline, consumer_header,
    dependency_leaf_code, dependency_tree_leaf, derive_help_messages, merged_cause, plan_resolved,
};

use super::support::{self, field_leaf, trait_hop};

/// The plan's `root cause:` note as text. The plan carries the note unrendered so the emitter can
/// elide it against what earlier diagnostics drew; a test has no such context, so it renders alone.
fn notes(plan: &cargo_cgp_error_processing::DiagnosisPlan) -> Vec<String> {
    plan.note.render(&mut std::collections::HashSet::new())
}

/// Indent every line by the two spaces the note wording nests a dependency chain under its
/// `this is required through the dependency chain:` heading with.
fn indent2(chain: &str) -> String {
    chain
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A consumer-trait-impl hop node.
fn consumer(trait_ref: &str, context: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Consumer {
        trait_ref: trait_ref.to_owned(),
        context: context.to_owned(),
    })
}

/// A provider-trait-impl hop node.
fn provider(trait_ref: &str, context: &str, provider: &str) -> ChainNode {
    ChainNode::Hop(DepNode::Provider {
        trait_ref: trait_ref.to_owned(),
        context: context.to_owned(),
        provider: provider.to_owned(),
    })
}

/// A redirect-lookup hop node (no dispatched key).
fn redirect(path: &str, table: &str) -> ChainNode {
    support::redirect(path, table, "")
}

/// The rendered dependency chain for a single path — the graph over just that path.
fn render_path(path: &[ChainNode]) -> String {
    DependencyGraph::from_paths(&[path.to_vec()]).render()
}

/// A cause with one path down to `leaf` (the terminal node is `leaf` itself).
fn one_path(leaf: Leaf, hops: Vec<ChainNode>) -> Cause {
    let mut path = hops;
    path.push(ChainNode::Leaf(leaf.clone()));
    Cause {
        leaf,
        paths: vec![path],
    }
}

/// A `Resolved` for the common case — CGP consumer traits failing on the context itself, so both
/// `consumers_are_cgp` and `subject_is_context` are `true`. The wrapper-header tests that vary
/// those flags build `Resolved` explicitly instead.
fn cgp_resolved(context: &str, consumers: &[&str], causes: Vec<Cause>) -> Resolved {
    Resolved {
        context: context.to_owned(),
        consumers: consumers.iter().map(|c| (*c).to_owned()).collect(),
        consumers_are_cgp: true,
        subject_is_context: true,
        causes: causes.into_iter().collect(),
    }
}

/// An empty name map — the categorized-header paths worded from the resolution never consult it.
fn empty_names() -> ComponentNameMap {
    ComponentNameMap::new(HashMap::new)
}

/// A name map carrying one provider trait, for the `IsProviderFor` text-rewrite header path.
fn foo_names() -> HashMap<String, ComponentTraitNames> {
    let mut map = HashMap::new();
    map.insert(
        "FooComponent".to_owned(),
        ComponentTraitNames {
            consumer: "CanFoo".to_owned(),
            provider: "Fooer".to_owned(),
            params: 0,
        },
    );
    map
}

fn missing_field_leaf() -> Leaf {
    field_leaf("height", "Rectangle", FieldIssue::Missing)
}

fn missing_field_cause() -> Cause {
    one_path(
        missing_field_leaf(),
        vec![consumer("CanCalculateArea", "Rectangle")],
    )
}

/// One field-missing cause whose path shares a `getter` hop's prefix with its sibling — the branch
/// point of the merge test below.
fn field_cause(getter: &str, field: &str) -> Cause {
    one_path(
        field_leaf(field, "Person", FieldIssue::Missing),
        vec![
            consumer("CanGreet", "Person"),
            provider("Greeter", "Person", "GreetFullName"),
            trait_hop(getter, "Person"),
        ],
    )
}
