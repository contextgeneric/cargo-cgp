//! Merging causes that name the same root cause into one.

use crate::diagnosis::resolved::Cause;

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
