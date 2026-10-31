//! The displayed headline of a resolved failure: its cause, after the class code.

use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::resolved::{Cause, Resolved};
use crate::diagnosis::wording::lead::root_cause_lead;
use crate::diagnosis::wording::{class_code, class_sentence, quoted_list};

/// The headline a resolved block displays, after its class `code`. One cause is stated as itself;
/// several causes of one kind on one owner are stated together ([`merged_cause`]); causes too mixed
/// to state at once fall back to the block's `class_sentence` with their count, and the note lists
/// each. The class sentence stays the block's internal headline either way, the one the
/// de-duplication ledger matches a declined re-report on.
pub fn cause_headline(code: &str, class_sentence: &str, causes: &[Cause]) -> String {
    match merged_cause(causes) {
        Some(cause) => format!("[{code}] {cause}"),
        None => format!("[{code}] {class_sentence}: {} root causes", causes.len()),
    }
}

/// The most items a merged headline names before it counts the rest (see [`merged_cause`]).
pub const MERGED_CAUSE_NAMES: usize = 3;

/// One sentence stating every cause, when they can share one: a single cause, or several of one
/// kind on one owner (fields missing from one struct, wiring missing from one context, entries
/// missing from one provider's table, data traits one type lacks). A long list names its first
/// [`MERGED_CAUSE_NAMES`] items and counts the rest, which the note lists in full. `None` for no
/// causes or a mix.
pub fn merged_cause(causes: &[Cause]) -> Option<String> {
    match causes {
        [] => None,
        [only] => Some(root_cause_lead(&only.leaf)),
        [first, ..] => {
            let (kind, owner, _) = mergeable(&first.leaf)?;
            let mut items: Vec<String> = Vec::new();
            for cause in causes {
                let (other_kind, other_owner, item) = mergeable(&cause.leaf)?;
                if other_kind != kind || other_owner != owner {
                    return None;
                }
                items.push(item.to_owned());
            }
            let items = capped_list(&items);
            Some(match kind {
                MergeKind::Field => format!("missing fields {items} on `{owner}`"),
                MergeKind::Wiring => format!("missing wiring for {items} on `{owner}`"),
                MergeKind::Entry => {
                    format!("missing entries for {items} in provider `{owner}`")
                }
                MergeKind::DataTrait => format!("`{owner}` does not implement {items}"),
            })
        }
    }
}

/// A [`quoted_list`](fn@quoted_list) of `items`, or its first [`MERGED_CAUSE_NAMES`] items and a count of the rest
/// when the list is longer. A list one item past the cap is named in full, since its count would
/// take as much room as the item.
fn capped_list(items: &[String]) -> String {
    if items.len() <= MERGED_CAUSE_NAMES + 1 {
        return quoted_list(items);
    }
    let named: Vec<String> = items[..MERGED_CAUSE_NAMES]
        .iter()
        .map(|item| format!("`{item}`"))
        .collect();
    format!(
        "{}, and {} more",
        named.join(", "),
        items.len() - MERGED_CAUSE_NAMES
    )
}

/// The label on each failing site of a coalesced block after the first, which is the block's caret.
/// An editor publishes one diagnostic per primary span, so the other sites ride as labelled
/// secondary spans and the block stays one diagnostic.
pub const COALESCED_SITE_LABEL: &str = "also required here";

/// The headline of a block that coalesces several resolved failures sharing a root cause: the
/// first member's class code, then the merged cause, or, for mixed causes, every member's class
/// sentence with the cause count.
pub fn coalesced_headline(members: &[&Resolved], causes: &[Cause]) -> String {
    let code = members
        .first()
        .map(|first| class_code(first))
        .unwrap_or_default();
    cause_headline(code, &coalesced_class_sentence(members), causes)
}

/// The class sentence of a coalesced block: one clause per distinct subject, in arrival order,
/// each naming every trait its members fail to implement there.
fn coalesced_class_sentence(members: &[&Resolved]) -> String {
    let mut subjects: Vec<Resolved> = Vec::new();
    for member in members {
        let same_subject = |subject: &&mut Resolved| {
            subject.context == member.context
                && subject.subject_is_context == member.subject_is_context
                && subject.class == member.class
        };
        match subjects.iter_mut().find(|subject| same_subject(subject)) {
            Some(subject) => {
                for consumer in &member.consumers {
                    if !subject.consumers.contains(consumer) {
                        subject.consumers.push(consumer.clone());
                    }
                }
            }
            None => subjects.push(Resolved {
                causes: Default::default(),
                ..(*member).clone()
            }),
        }
    }
    let clauses: Vec<String> = subjects.iter().map(class_sentence).collect();
    match clauses.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The kinds of missing thing that several of, on one owner, state as one: fields, wiring entries,
/// table entries, and the data traits a type lacks.
#[derive(PartialEq, Eq)]
enum MergeKind {
    Field,
    Wiring,
    Entry,
    DataTrait,
}

/// A cause's merge kind, its owner, and the item it names, or `None` for a cause that is never
/// stated together with another.
fn mergeable(leaf: &Leaf) -> Option<(MergeKind, &str, &str)> {
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
            ..
        } => Some((MergeKind::Field, owner, name)),
        Leaf::MissingWiring {
            component, owner, ..
        } => Some((MergeKind::Wiring, owner, component)),
        Leaf::MissingRedirectWiring { path, context, .. } => {
            Some((MergeKind::Wiring, context, path))
        }
        Leaf::MissingDispatchEntry { key, table } => Some((MergeKind::Entry, table, key)),
        Leaf::DataTraitUnimplemented {
            owner, trait_name, ..
        } => Some((MergeKind::DataTrait, owner, trait_name)),
        _ => None,
    }
}
