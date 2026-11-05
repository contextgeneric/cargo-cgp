//! The `help` messages a resolved failure carries: the fixes its causes call for, merged.

use crate::diagnosis::resolved::Cause;
use crate::diagnosis::wording::fix_kind::FixKind;
use crate::diagnosis::wording::fixes::fixes_for;
use crate::diagnosis::wording::quoted_list;

/// Every `help` a resolved failure's causes call for, in the order the causes are listed. The fixes
/// of one kind on one owner merge into one `help`: the fields to add to one struct, the derive one
/// struct needs, the components to wire on one context, the entries to add to one table, and the
/// data derives one type needs (named in one attribute). Every other fix is worded whole: a near
/// miss, a tuple field's tag, an abstract type to reconcile, a non-provider's adapter, component, or
/// core provider, a hand-written wiring to replace, a bound to add, and a method to call. A missing
/// field is named with its type when its requirement states one, and gets no help when adding it is
/// not confirmed to fix the cause (see [`FieldFix`](crate::FieldFix)); a key the reader could not
/// paste gets none either. Both the streaming plan and the emitter's coalesced block build their
/// `help`s through this, so a block that merges several consumers carries the same fixes as the
/// per-consumer one it replaces.
///
/// Postcondition: a near miss, a similar name the program wrote, comes before the fix of its
/// cause, also when that fix merges into a `help` an earlier cause opened.
pub fn fix_help_messages(causes: &[Cause]) -> Vec<String> {
    // One group per help: its kind and owner, and the items it names in first-seen order. A whole
    // fix's owner is its text.
    let mut groups: Vec<(FixKind, String, Vec<String>)> = Vec::new();
    for cause in causes {
        // The whole fixes `fixes_for` lists before a cause's mergeable fix are its near misses,
        // placed just before that fix's group wherever the group sits.
        let mut near_misses: Vec<String> = Vec::new();
        for (kind, owner, item) in fixes_for(&cause.leaf) {
            if kind == FixKind::Whole {
                near_misses.push(owner);
                continue;
            }
            let found = groups
                .iter()
                .position(|(seen_kind, seen_owner, _)| *seen_kind == kind && *seen_owner == owner);
            let at = found.unwrap_or_else(|| {
                groups.push((kind, owner, Vec::new()));
                groups.len() - 1
            });
            let at = insert_whole_before(&mut groups, at, near_misses.drain(..));
            if let Some(item) = item
                && !groups[at].2.contains(&item)
            {
                groups[at].2.push(item);
            }
        }
        let end = groups.len();
        insert_whole_before(&mut groups, end, near_misses);
    }
    groups
        .into_iter()
        .map(|(kind, owner, items)| {
            let one = items.len() == 1;
            let derives = items.join(", ");
            let items = quoted_list(&items);
            match kind {
                FixKind::AddFields => {
                    let noun = if one { "field" } else { "fields" };
                    format!("add the {noun} {items} to `{owner}`")
                }
                FixKind::Derive => format!("add `#[derive(HasField)]` to `{owner}`"),
                FixKind::Wire => {
                    let provider = if one { "a provider" } else { "providers" };
                    format!("wire {items} to {provider} in the wiring of `{owner}`")
                }
                FixKind::AddEntries => {
                    let entry = if one { "an entry" } else { "entries" };
                    format!("add {entry} for {items} to `{owner}`")
                }
                FixKind::DeriveData => format!("add `#[derive({derives})]` to `{owner}`"),
                FixKind::Whole => owner,
            }
        })
        .collect()
}

/// Insert each whole `help` in `texts` that `groups` does not hold yet at index `at`, in order, and
/// return the index the group at `at` moved to.
fn insert_whole_before(
    groups: &mut Vec<(FixKind, String, Vec<String>)>,
    mut at: usize,
    texts: impl IntoIterator<Item = String>,
) -> usize {
    for text in texts {
        let seen = groups
            .iter()
            .any(|(kind, owner, _)| *kind == FixKind::Whole && *owner == text);
        if !seen {
            groups.insert(at, (FixKind::Whole, text, Vec::new()));
            at += 1;
        }
    }
    at
}
