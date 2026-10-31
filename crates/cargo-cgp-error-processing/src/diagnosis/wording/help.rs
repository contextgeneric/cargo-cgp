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
/// paste gets none either. A near miss, a similar name the program wrote, comes before the fix of
/// its cause. Both the streaming plan and the emitter's coalesced block build their `help`s through
/// this, so a block that merges several consumers carries the same fixes as the per-consumer one it
/// replaces.
pub fn fix_help_messages(causes: &[Cause]) -> Vec<String> {
    // One group per help: its kind and owner, and the items it names in first-seen order.
    let mut groups: Vec<(FixKind, String, Vec<String>)> = Vec::new();
    for (kind, owner, item) in causes.iter().flat_map(|cause| fixes_for(&cause.leaf)) {
        let group = match groups
            .iter_mut()
            .find(|(seen_kind, seen_owner, _)| *seen_kind == kind && *seen_owner == owner)
        {
            Some(group) => group,
            None => {
                groups.push((kind, owner, Vec::new()));
                groups.last_mut().expect("a group was just pushed")
            }
        };
        if let Some(item) = item
            && !group.2.contains(&item)
        {
            group.2.push(item);
        }
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
