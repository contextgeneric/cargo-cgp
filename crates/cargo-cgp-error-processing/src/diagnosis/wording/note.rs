//! The `root cause:` note body, rendered over one dependency graph.

use std::collections::HashSet;

use crate::diagnosis::graph::DependencyGraph;
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::node::ChainNode;
use crate::diagnosis::resolved::Cause;
use crate::diagnosis::wording::lead::{root_cause_code, root_cause_lead};

/// The `root cause:` note(s) for a resolved failure. Every path of every cause is folded into one
/// [`DependencyGraph`], which merges the nodes they share and renders the whole as a `cargo
/// tree`-style diagram; the heading then names the distinct root cause(s) above it. The result is a
/// single note (or none, for no causes), since the graph already fuses what the old per-root grouping
/// kept apart.
///
/// `header_leaf` is the leaf the diagnostic's main message already states in full, if any — its lead
/// is then dropped so the note does not repeat the header. A caller that rewords the header itself,
/// as the emitter's coalesced block does, passes `None`: its header names the affected consumers
/// rather than the cause, so every lead is worth keeping.
pub fn cause_notes(causes: &[Cause], header_leaf: Option<&Leaf>) -> Vec<String> {
    cause_notes_seen(causes, header_leaf, &mut HashSet::new())
}

/// [`cause_notes`] against a `seen` set shared with the other notes of one compilation, so a subtree
/// an earlier note already drew is `(*)`-referenced here rather than repeated in full. See
/// [`DependencyGraph::render_seen`].
pub fn cause_notes_seen(
    causes: &[Cause],
    header_leaf: Option<&Leaf>,
    seen: &mut HashSet<ChainNode>,
) -> Vec<String> {
    if causes.is_empty() {
        return Vec::new();
    }
    let graph = causes_graph(causes);

    // The distinct root causes, in first-seen order — a leaf reached by several paths is named once.
    let mut leaves: Vec<&Leaf> = Vec::new();
    for cause in causes {
        if !leaves.iter().any(|leaf| **leaf == cause.leaf) {
            leaves.push(&cause.leaf);
        }
    }

    // Every root of this graph was drawn by an earlier note, so eliding against that note has nothing
    // to *trim* — it would remove the whole derivation and leave the block asserting a cause with no
    // account of where it came from. The cross-block elision exists to drop a long shared tail hanging
    // under a block's own distinct prefix; a block with no distinct prefix at all is the degenerate
    // case, not the case it was built for. Such a block has already survived de-duplication, so it is a
    // *different* failure that happens to run through ground another one covered, and it earns its own
    // chain: render it against a fresh set, ignoring what came before. The cost is bounded — a
    // wholly-contained graph is by construction no larger than the one that contains it — and the
    // nodes still join `seen`, so later notes elide against this block as usual.
    let chain = if graph.fully_elided_by(seen) {
        let mut fresh = HashSet::new();
        let rendered = graph.render_seen(&mut fresh);
        seen.extend(fresh);
        rendered
    } else {
        graph.render_seen(seen)
    };
    let note = match leaves.as_slice() {
        [only] => leaf_lead(only, header_leaf)
            .map(|lead| format!("{lead}\n{}", chain_heading(&chain)))
            .unwrap_or_else(|| chain_heading(&chain)),
        many => format!(
            "root causes:\n{}\n{}",
            leaf_list(many),
            chain_heading(&chain)
        ),
    };
    vec![note]
}

/// The note body for one cause on its own — its lead over its own paths' graph. Kept for callers and
/// tests that word a single cause; equivalent to [`cause_notes`] over a one-element slice.
pub fn cause_note(cause: &Cause, header_leaf: Option<&Leaf>) -> String {
    let chain = causes_graph(std::slice::from_ref(cause)).render();
    match leaf_lead(&cause.leaf, header_leaf) {
        Some(lead) => format!("{lead}\n{}", chain_heading(&chain)),
        None => chain_heading(&chain),
    }
}

/// The dependency graph built from every path of every cause.
fn causes_graph(causes: &[Cause]) -> DependencyGraph {
    let paths: Vec<Vec<ChainNode>> = causes
        .iter()
        .flat_map(|cause| cause.paths.iter().cloned())
        .collect();
    DependencyGraph::from_paths(&paths)
}

/// The `- [code] lead` bullet per distinct root cause, one per line, for a `root causes:` heading.
fn leaf_list(leaves: &[&Leaf]) -> String {
    leaves
        .iter()
        .map(|leaf| {
            format!(
                "  - [{code}] {lead}",
                code = root_cause_code(leaf),
                lead = root_cause_lead(leaf),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The `root cause: [code] lead` line for a single leaf, or `None` when the note should carry the
/// chain alone because the main message already states this very leaf — a rewritten mismatch header
/// naming the type, or a kept rustc header restating the ordinary bound the walk descended to.
///
/// The test is whether the header states *this* leaf, not what kind of leaf it is. Keying on the
/// kind instead would drop a mismatch's lead even under a header that never mentioned it — as the
/// emitter's coalesced block produces, where the header lists the affected consumers and the lead is
/// the only place above the tree the cause appears.
fn leaf_lead(leaf: &Leaf, header_leaf: Option<&Leaf>) -> Option<String> {
    if header_leaf == Some(leaf) {
        return None;
    }
    Some(format!(
        "root cause: [{code}] {lead}",
        code = root_cause_code(leaf),
        lead = root_cause_lead(leaf),
    ))
}

/// The `this is required through the dependency chain:` heading with the rendered chain nested two
/// spaces beneath it.
fn chain_heading(chain: &str) -> String {
    format!(
        "this is required through the dependency chain:\n{}",
        indent_chain(chain)
    )
}

/// Indent every line of a rendered dependency chain by two spaces, so it nests under the heading.
fn indent_chain(chain: &str) -> String {
    chain
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
