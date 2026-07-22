//! The `root cause:` note body, rendered over one dependency graph.

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
pub fn cause_notes(causes: &[Cause], header_bound: Option<&str>) -> Vec<String> {
    if causes.is_empty() {
        return Vec::new();
    }
    let chain = render_causes(causes);

    // The distinct root causes, in first-seen order — a leaf reached by several paths is named once.
    let mut leaves: Vec<&Leaf> = Vec::new();
    for cause in causes {
        if !leaves.iter().any(|leaf| **leaf == cause.leaf) {
            leaves.push(&cause.leaf);
        }
    }
    let note = match leaves.as_slice() {
        [only] => leaf_lead(only, header_bound)
            .map(|lead| format!("{lead}\n{}", chain_heading(&chain)))
            .unwrap_or_else(|| chain_heading(&chain)),
        many => {
            let list = many
                .iter()
                .map(|leaf| {
                    format!(
                        "  - [{code}] {lead}",
                        code = root_cause_code(leaf),
                        lead = root_cause_lead(leaf),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("root causes:\n{list}\n{}", chain_heading(&chain))
        }
    };
    vec![note]
}

/// The note body for one cause on its own — its lead over its own paths' graph. Kept for callers and
/// tests that word a single cause; equivalent to [`cause_notes`] over a one-element slice.
pub fn cause_note(cause: &Cause, header_bound: Option<&str>) -> String {
    let chain = render_causes(std::slice::from_ref(cause));
    match leaf_lead(&cause.leaf, header_bound) {
        Some(lead) => format!("{lead}\n{}", chain_heading(&chain)),
        None => chain_heading(&chain),
    }
}

/// Render the dependency graph built from every path of every cause.
fn render_causes(causes: &[Cause]) -> String {
    let paths: Vec<Vec<ChainNode>> = causes
        .iter()
        .flat_map(|cause| cause.paths.iter().cloned())
        .collect();
    DependencyGraph::from_paths(&paths).render()
}

/// The `root cause: [code] lead` line for a single leaf, or `None` when the note should carry the
/// chain alone — a field-type mismatch (whose `[CGP-E003]` header already states it in full), or an
/// ordinary bound the kept main message (`header_bound`) already restates.
fn leaf_lead(leaf: &Leaf, header_bound: Option<&str>) -> Option<String> {
    if let Leaf::FieldTypeMismatch { .. } = leaf {
        return None;
    }
    if let (Some(bound), Leaf::Bound { summary }) = (header_bound, leaf)
        && summary == bound
    {
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
