//! Grouping a compilation's coalescible failures by the root causes they share.

use std::collections::HashMap;

use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::cause_keys;

/// Partition failures into groups that **share a root cause**, so the emitter can coalesce each
/// group into one headline listing every affected consumer. Returns one `Vec` of indices into
/// `resolveds` per group, each group in arrival order and the groups themselves ordered by their
/// first member — so the caller emits each group where its first member arrived and overall ordering
/// is preserved.
///
/// Two failures group when they share *at least one* [cause key](cause_keys), and grouping is
/// transitive: the groups are the connected components of the "shares a cause" relation. Demanding
/// two *identical* cause sets instead is what left one mistake reported as several blocks. CGP wiring
/// is lazy, so a single omission surfaces at several depths, and each depth sees a different subset of
/// its causes: a `check_components!` entry stops at the first unmet leaf on its own branch, while a
/// use-site call walks every wired component and reaches them all. Those subsets overlap without ever
/// being equal, so no two of them grouped — and the block whose causes were the *union* of the others
/// fared worst, since its chain was then fully elided against the blocks that had already drawn every
/// one of its roots, leaving a bare `root causes:` list with no dependency chain at all.
///
/// Transitivity is what makes the relation a partition rather than an ambiguous overlap graph: a
/// failure whose causes are covered by two others would otherwise have no single group to join.
/// Grouping this way also earns an invariant the exact-match key could not: **no two coalesced blocks
/// share a root cause**, so one mistake is stated in exactly one block, and two blocks can no longer
/// have the same top-level roots by way of a shared cause — which is what made the whole-chain elision
/// degenerate in the first place.
///
/// A failure with no causes shares nothing and forms its own group.
pub fn group_by_shared_cause(resolveds: &[&Resolved]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..resolveds.len()).collect();

    // Union every failure carrying a cause key with the first failure that carried it, so each key
    // links its holders into one component.
    let mut first_holder: HashMap<String, usize> = HashMap::new();
    for (index, resolved) in resolveds.iter().enumerate() {
        for key in cause_keys(resolved) {
            match first_holder.get(&key) {
                Some(&holder) => union(&mut parent, index, holder),
                None => {
                    first_holder.insert(key, index);
                }
            }
        }
    }

    // Collect the components, each in arrival order, ordered by their first member.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut group_of_root: HashMap<usize, usize> = HashMap::new();
    for index in 0..resolveds.len() {
        let root = find(&mut parent, index);
        match group_of_root.get(&root) {
            Some(&group) => groups[group].push(index),
            None => {
                group_of_root.insert(root, groups.len());
                groups.push(vec![index]);
            }
        }
    }
    groups
}

/// The representative of `index`'s component, with path halving so repeated lookups stay near-flat.
fn find(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        let grandparent = parent[parent[index]];
        parent[index] = grandparent;
        index = grandparent;
    }
    index
}

/// Merge the components of `left` and `right`.
fn union(parent: &mut [usize], left: usize, right: usize) {
    let (left, right) = (find(parent, left), find(parent, right));
    if left != right {
        parent[left] = right;
    }
}
