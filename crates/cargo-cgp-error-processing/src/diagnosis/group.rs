//! Grouping a compilation's coalescible failures by the root causes they share.

use std::collections::HashMap;

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Resolved;

/// Partition failures into groups that **share a root cause**, transitively, so the emitter can
/// coalesce each group into one block. Returns one `Vec` of indices into `resolveds` per group,
/// each in arrival order, the groups ordered by their first member.
///
/// Postcondition: no two groups share a root cause, a cause being identified by its [`Leaf`], and
/// an ordinary bound together with the context it was reached from.
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/driver/program.md` for why grouping is
/// transitive and keys on the leaf rather than its wording.
pub fn group_by_shared_cause(resolveds: &[&Resolved]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..resolveds.len()).collect();

    // Union every failure carrying a cause with the first failure that carried it, so each cause
    // links its holders into one component.
    let mut first_holder: HashMap<(Option<&str>, &Leaf), usize> = HashMap::new();
    for (index, resolved) in resolveds.iter().enumerate() {
        for cause in resolved.causes.iter() {
            let context =
                matches!(cause.leaf, Leaf::Bound { .. }).then_some(resolved.context.as_str());
            let key = (context, &cause.leaf);
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
