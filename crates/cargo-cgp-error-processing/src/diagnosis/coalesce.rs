//! Coalescing root causes that share one fix into a single cause.

use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::resolved::Cause;
use crate::tree::merge_dependency_forest;

/// Coalesce causes that are each a *present-but-underived* field on the same struct into one
/// [`Leaf::UnderivedFields`] cause, since `#[derive(HasField)]` derives an impl for every field —
/// several underived fields on one struct are one mistake with one fix, and listing them as
/// separate root causes overstates the work. The merged cause keeps a single tree
/// ([`merge_dependency_forest`]) whose branches still end at the per-field leaves, so each field
/// stays visible in the chain, and it takes the position of the group's first member.
///
/// The coalescing keys on [`FieldIssue::Present`] deliberately: a *genuinely absent* field
/// ([`FieldIssue::Missing`]) is its own fix (add the field), so a struct missing several fields
/// correctly keeps several causes — and a fieldless struct's derive, which emits no impls, reads
/// as absent fields, not underived ones. Underived fields on *different* structs are different
/// fixes and stay apart, as does a lone underived field, which keeps its more specific
/// single-field wording. Causes whose trees share no root are left alone too, since their merge
/// would yield a forest rather than one chain.
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
        let merged = merge_dependency_forest(
            &group
                .iter()
                .map(|cause| cause.tree.clone())
                .collect::<Vec<_>>(),
        );
        // A lone underived field keeps its single-field wording, and a group whose chains share
        // no root would merge into a forest rather than one tree — keep those apart.
        let [tree] = merged.as_slice() else {
            out.push(cause.clone());
            continue;
        };
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
        out.push(Cause {
            leaf: Leaf::UnderivedFields {
                names,
                owner: owner.clone(),
            },
            tree: tree.clone(),
        });
        coalesced.push(owner);
    }
    out
}
