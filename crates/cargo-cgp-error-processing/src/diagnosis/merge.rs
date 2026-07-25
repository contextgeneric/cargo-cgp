//! Merging causes that name the same root cause into one, and heading them with an enclosing hop.

use crate::diagnosis::node::{ChainNode, DepNode};
use crate::diagnosis::resolved::Cause;

/// Head every path of every cause with `node`, then re-merge by leaf — the operation an anchor
/// performs when it hangs a recovered CGP chain beneath the trait the programmer actually wrote (the
/// wrapper an `impl` block names, say), so the tree reads from their own code down to the cause.
///
/// Prepending and merging are one operation rather than two deliberately. Both anchors that do this
/// used to spell it out themselves and in *opposite* orders — one prepended then merged, the other
/// merged then prepended — which was correct only because `node` is a single constant hop that cannot
/// change any leaf's identity. Nothing stated that precondition, so the divergence read as though one
/// of the two must be wrong. One function fixes the order once and makes the question disappear.
pub fn prepend_hop(causes: &[Cause], node: &DepNode) -> Vec<Cause> {
    let headed: Vec<Cause> = causes
        .iter()
        .map(|cause| Cause {
            leaf: cause.leaf.clone(),
            paths: cause
                .paths
                .iter()
                .map(|path| {
                    let mut path = path.clone();
                    path.insert(0, ChainNode::Hop(node.clone()));
                    path
                })
                .collect(),
        })
        .collect();
    merge_causes_by_leaf(&headed)
}

/// Merge causes naming the *same* leaf into one cause holding every path that reaches it,
/// restoring the *one cause per distinct leaf* invariant [`Cause`] documents.
///
/// A single resolution already upholds that invariant — the walk groups its sub-paths by leaf — so
/// this exists for a caller that **unions the causes of several resolutions**: the emitter's
/// coalesced block, which merges the consumer failures that share one root cause. Each member
/// carries its own copy of that shared cause, and left unmerged the duplicates make every
/// downstream reader count one mistake several times. The visible casualty is
/// [`coalesce_underived_fields`](super::coalesce_underived_fields), which reads several
/// underived-field causes on one struct as several fields and lists them all: three consumers
/// failing on one underived `name` field produced `` the fields `name`, `name`, and `name` ``
/// where the single-field wording is meant.
///
/// Paths are concatenated in first-seen order, with an exact repeat dropped. That is a saving
/// rather than a change: the [dependency graph](crate::DependencyGraph) merges paths by structural
/// identity anyway, so a duplicate path renders identically either way.
pub fn merge_causes_by_leaf(causes: &[Cause]) -> Vec<Cause> {
    let mut merged: Vec<Cause> = Vec::new();

    for cause in causes {
        let Some(existing) = merged.iter_mut().find(|seen| seen.leaf == cause.leaf) else {
            merged.push(cause.clone());
            continue;
        };
        for path in &cause.paths {
            if !existing.paths.contains(path) {
                existing.paths.push(path.clone());
            }
        }
    }

    merged
}
