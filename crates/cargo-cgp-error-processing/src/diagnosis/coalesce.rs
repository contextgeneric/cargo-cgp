//! Coalescing root causes that share one fix into a single cause.

use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::resolved::Cause;

/// Coalesce causes that are each a *present-but-underived* field on the same struct into one
/// [`Leaf::UnderivedFields`] cause, keeping every field's path.
///
/// See cgp-knowledge-base/cargo-cgp/implementation/dependency-graph-rendering.md for why only a
/// present field on one struct merges.
pub fn coalesce_underived_fields(causes: &[Cause]) -> Vec<Cause> {
    let mut out: Vec<Cause> = Vec::new();
    // The owners already coalesced, so the group is emitted once, at its first member's position.
    let mut coalesced: Vec<&str> = Vec::new();

    for cause in causes {
        let Leaf::Field {
            owner,
            issue: FieldIssue::Present,
            ..
        } = &cause.leaf
        else {
            out.push(cause.clone());
            continue;
        };
        if coalesced.iter().any(|done| done == owner) {
            continue;
        }

        let group: Vec<&Cause> = causes
            .iter()
            .filter(|candidate| {
                matches!(
                    &candidate.leaf,
                    Leaf::Field { owner: candidate_owner, issue: FieldIssue::Present, .. }
                        if candidate_owner == owner
                )
            })
            .collect();
        // A lone underived field keeps its single-field wording.
        if group.len() < 2 {
            out.push(cause.clone());
            continue;
        }

        let names = group
            .iter()
            .filter_map(|cause| match &cause.leaf {
                Leaf::Field { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        // Keep every field's path, so the graph still branches to each per-field leaf.
        let paths = group
            .iter()
            .flat_map(|cause| cause.paths.iter().cloned())
            .collect();
        out.push(Cause {
            leaf: Leaf::UnderivedFields {
                names,
                owner: owner.clone(),
            },
            paths,
        });
        coalesced.push(owner);
    }
    out
}
