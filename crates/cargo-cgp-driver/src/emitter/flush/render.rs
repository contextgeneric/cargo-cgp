//! Rendering the whole buffer, in arrival order.

use std::collections::{HashMap, HashSet};

use cargo_cgp_error_processing::{AmbiguousNames, Resolved, group_by_shared_cause};
use rustc_errors::DiagInner;
use rustc_span::source_map::SourceMap;

use crate::emitter::buffer::BufEntry;
use crate::emitter::flush::{ChainNodeSet, insert_note, merged_diag, rebuild_explain_footers};
use crate::emitter::repeats::merge_repeats;

/// The coalescible resolutions of a buffer, grouped by the root causes they share.
pub struct CoalescedGroups {
    /// Each group's members, as buffer indices in arrival order.
    pub groups: Vec<Vec<usize>>,
    /// The group of each coalescible entry, by its buffer index.
    pub group_of: HashMap<usize, usize>,
}

/// Render the buffered diagnostics for the flush, coalescing each group of coalescible failures
/// that share a root cause into one block at the position of its first member. `ambiguous` keeps
/// the qualifiers the post-processing of a rewritten message keeps, and `repeats`, the source map
/// when the crate is known to use CGP, merges a diagnostic rustc reported twice, so a plain Rust
/// error is never touched.
pub fn render(
    buffer: &[BufEntry],
    ambiguous: &AmbiguousNames,
    repeats: Option<&SourceMap>,
) -> Vec<DiagInner> {
    let groups = coalesced_groups(buffer);
    let mut to_emit = render_blocks(buffer, &groups, ambiguous);
    if let Some(source_map) = repeats {
        to_emit = merge_repeats(to_emit, source_map);
    }
    // Now that the surviving set is final, make the "detailed explanations" footer agree with it.
    rebuild_explain_footers(&mut to_emit);
    to_emit
}

/// The coalescible entries in arrival order, grouped by the root causes they share. Grouping on
/// shared causes rather than on one whole-failure key keeps a single mistake in a single block: the
/// depths it surfaces at each see a different subset of its causes, so no two of them ever carry
/// equal keys (see [`group_by_shared_cause`]).
pub fn coalesced_groups(buffer: &[BufEntry]) -> CoalescedGroups {
    let positions: Vec<usize> = buffer
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match entry {
            BufEntry::Resolved(entry) if entry.coalescible => Some(index),
            _ => None,
        })
        .collect();
    let resolveds: Vec<&Resolved> = positions
        .iter()
        .map(|&index| match &buffer[index] {
            BufEntry::Resolved(entry) => &entry.resolved,
            _ => unreachable!("indices came from the coalescible entries"),
        })
        .collect();
    let groups: Vec<Vec<usize>> = group_by_shared_cause(&resolveds)
        .into_iter()
        .map(|group| group.into_iter().map(|member| positions[member]).collect())
        .collect();
    let group_of: HashMap<usize, usize> = groups
        .iter()
        .enumerate()
        .flat_map(|(group, members)| members.iter().map(move |&index| (index, group)))
        .collect();
    CoalescedGroups { groups, group_of }
}

/// Each buffered entry as the diagnostic to emit, in arrival order: a plain entry as it stands, a
/// resolution with its note rendered, and a group of several resolutions as one merged block at its
/// first member's position. The subtrees drawn so far are threaded through every block in emission
/// order, so a later one truncates at what an earlier one already showed; this is why notes are
/// rendered here rather than on arrival, since only now is that order known.
fn render_blocks(
    buffer: &[BufEntry],
    groups: &CoalescedGroups,
    ambiguous: &AmbiguousNames,
) -> Vec<DiagInner> {
    let mut seen = ChainNodeSet::new();
    let mut emitted: HashSet<usize> = HashSet::new();
    let mut to_emit: Vec<DiagInner> = Vec::new();
    for (index, entry) in buffer.iter().enumerate() {
        match entry {
            BufEntry::Plain(diag) => to_emit.push((**diag).clone()),
            BufEntry::Resolved(entry) => {
                let Some(&group) = groups.group_of.get(&index) else {
                    // A resolution that never coalesces: render its own note and emit it here.
                    let mut diag = entry.diag.clone();
                    insert_note(&mut diag, &entry.note, &mut seen, ambiguous);
                    to_emit.push(diag);
                    continue;
                };
                if !emitted.insert(group) {
                    continue;
                }
                let members = &groups.groups[group];
                if members.len() == 1 {
                    // A group's first member is where it is emitted, so that member is `entry`.
                    let mut diag = entry.diag.clone();
                    insert_note(&mut diag, &entry.note, &mut seen, ambiguous);
                    to_emit.push(diag);
                } else {
                    let mut pairs: Vec<(&Resolved, &DiagInner)> = members
                        .iter()
                        .map(|&index| match &buffer[index] {
                            BufEntry::Resolved(member) => (&member.resolved, &member.diag),
                            _ => unreachable!("indices came from the coalescible entries"),
                        })
                        .collect();
                    // The block's code and first caret come from the member written first in the
                    // source, so the code names the class of the entry the reader sees first.
                    pairs.sort_by_key(|(_, diag)| diag.span.primary_span().map(|span| span.lo()));
                    let (resolveds, diags): (Vec<&Resolved>, Vec<&DiagInner>) =
                        pairs.into_iter().unzip();
                    to_emit.push(merged_diag(&resolveds, &diags, &mut seen, ambiguous));
                }
            }
        }
    }
    to_emit
}
