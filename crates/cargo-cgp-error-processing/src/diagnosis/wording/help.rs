//! The `help` messages a resolved failure carries: the fixes its causes call for.

use crate::diagnosis::leaf::{
    DeriveFix, EntryFix, FieldIssue, Leaf, MethodFix, NearMiss, SuggestedCall,
};
use crate::diagnosis::resolved::Cause;
use crate::diagnosis::wording::header::quoted_list;

/// The kinds of fix a `help` names. The fixes of one kind on one owner merge into one `help`.
#[derive(PartialEq, Eq)]
enum FixKind {
    /// Add the missing fields to a struct.
    AddFields,
    /// Derive `HasField` on the struct that carries the fields.
    Derive,
    /// Wire components or paths on a context.
    Wire,
    /// Add entries to a provider's table.
    AddEntries,
    /// Add data derives (`HasFields`, `ExtractField`, …) to a struct or enum.
    DeriveData,
    /// A `help` worded whole, such as a near miss or an abstract type to reconcile (its owner is
    /// the text itself).
    Whole,
}

/// The fixes a cause's leaf calls for, each as its kind, the owner it lands on, and the item it
/// names: a near miss first, since a slipped name is the likelier mistake, then the fix the cause
/// itself states. Empty for a cause whose fix the tool does not word.
fn fixes_for(leaf: &Leaf) -> Vec<(FixKind, String, Option<String>)> {
    let mut fixes = Vec::new();
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
            fix,
        } => {
            if let Some(similar) = &fix.similar {
                fixes.push(whole(format!(
                    "a similar field exists on `{owner}`: `{similar}`"
                )));
            }
            if let Some(tag) = &fix.index_tag {
                fixes.push(whole(format!(
                    "a field of the tuple struct `{owner}` is keyed by `{tag}`, not by `Symbol!(\"{name}\")`"
                )));
            }
            if fix.confirmed {
                let field = match &fix.ty {
                    Some(ty) => format!("{name}: {ty}"),
                    None => name.clone(),
                };
                fixes.push((FixKind::AddFields, owner.clone(), Some(field)));
            }
        }
        Leaf::Field {
            owner,
            issue: FieldIssue::Present,
            ..
        }
        | Leaf::UnderivedFields { owner, .. } => fixes.push((FixKind::Derive, owner.clone(), None)),
        Leaf::Field {
            issue: FieldIssue::PresentViaDeref { target },
            ..
        } => fixes.push((FixKind::Derive, target.clone(), None)),
        Leaf::MissingWiring {
            component,
            owner,
            similar,
        } => {
            fixes.extend(similar_key_fix(owner, similar));
            fixes.push((FixKind::Wire, owner.clone(), Some(component.clone())));
        }
        Leaf::MissingRedirectWiring {
            path,
            context,
            similar,
        } => {
            fixes.extend(similar_key_fix(context, similar));
            if is_typeable_key(path) {
                fixes.push((FixKind::Wire, context.clone(), Some(path.clone())));
            }
        }
        Leaf::MissingDispatchEntry { key, table } if is_typeable_key(key) => {
            fixes.push((FixKind::AddEntries, table.clone(), Some(key.clone())));
        }
        Leaf::WiringWithoutProviderFor {
            component,
            owner,
            delegate: EntryFix(Some(delegate)),
        } => fixes.push(whole(format!(
            "replace the hand-written `DelegateComponent` impl with `delegate_components! {{ {owner} {{ {component}: {delegate} }} }}`"
        ))),
        Leaf::UnboundedParameter {
            param,
            trait_name,
            item,
        } => fixes.push(whole(format!(
            "add the bound `{param}: {trait_name}` to `{item}`"
        ))),
        Leaf::AssocTypeMismatch { .. } => fixes.extend(assoc_mismatch_help(leaf).map(whole)),
        Leaf::MissingVariant { owner, similar, .. } => {
            if let Some(similar) = &similar.0 {
                fixes.push(whole(format!(
                    "a similar variant exists on `{owner}`: `{similar}`"
                )));
            }
        }
        Leaf::DataTraitUnimplemented {
            owner,
            derive: DeriveFix(Some(derive)),
            ..
        } => fixes.push((FixKind::DeriveData, owner.clone(), Some(derive.clone()))),
        Leaf::FieldNotOptional {
            convert: MethodFix(Some(call)),
            ..
        } => fixes.push(whole(format!(
            "call `{}` on the builder first to hold each field as optional{}",
            call.method,
            trait_note(call)
        ))),
        Leaf::PartialLacksDataTrait {
            kind,
            replacement: MethodFix(Some(call)),
            ..
        } => {
            if let Some(replaces) = &call.replaces {
                fixes.push(whole(format!(
                    "call `{}` in place of `{replaces}` on {}{}",
                    call.method,
                    kind.with_article(),
                    trait_note(call)
                )));
            }
        }
        _ => {}
    }
    fixes
}

/// The import a `help`'s method needs, `, with `use cgp::…::Trait;` in scope`, since a data
/// trait is not in the prelude and the compiler's own import suggestion can name a path that does
/// not compile. Empty when no import path is known.
fn trait_note(call: &SuggestedCall) -> String {
    match &call.trait_path {
        Some(path) => format!(", with `use {path};` in scope"),
        None => String::new(),
    }
}

/// A `help` worded whole, never merged with another.
fn whole(help: String) -> (FixKind, String, Option<String>) {
    (FixKind::Whole, help, None)
}

/// The near-miss `help` for a key missing from `context`'s wiring, when it wires a similar one.
fn similar_key_fix(context: &str, similar: &NearMiss) -> Option<(FixKind, String, Option<String>)> {
    let similar = similar.0.as_ref()?;
    Some(whole(format!(
        "a similar key exists in the wiring of `{context}`: `{similar}`"
    )))
}

/// Whether a wiring key reads as an entry the reader can paste. The key is the compiler's rendering
/// of its type, resugared only when the message is post-processed, and the compiler erases every
/// lifetime in it: a reference reads `&u64` and a lifetime argument `'_`, where the entry has to
/// name the lifetime (`<'a> @ItemEncoderComponent.&'a u64`). A `'_'` is a character of a `Symbol!`
/// spelled out, not a lifetime, so it does not count.
fn is_typeable_key(key: &str) -> bool {
    let erased_lifetime = key
        .match_indices("'_")
        .any(|(at, _)| !key[at + 2..].starts_with('\''));
    !key.contains('&') && !erased_lifetime
}

/// Every `help` a resolved failure's causes call for, in the order the causes are listed, one per
/// kind of fix on one owner: the fields to add to one struct, the derive one struct needs, the
/// components to wire on one context, the entries to add to one table, the data derives one type
/// needs (named in one attribute), and each abstract type to reconcile. A missing field is named with its type when its requirement states one, and gets no
/// help when adding it is not confirmed to fix the cause (see [`FieldFix`](crate::FieldFix)); a key
/// the reader could not paste gets none either. A missing field or key that has a near miss, a
/// similar name the program wrote, gets that named first. Both the streaming plan and the emitter's
/// coalesced block build their `help`s through this, so a block that merges several consumers
/// carries the same fixes as the per-consumer one it replaces.
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

/// The `help` for an abstract-type mismatch, naming the two ways to reconcile the two sides: bind
/// the component to the type the provider requires, or relax the provider to work with the one the
/// context supplies. Only a CGP abstract-type component earns a `help`: its concrete type is a
/// wiring choice, so there is a specific entry to change (`UseType<T>`, the provider
/// [`#[cgp_type]`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/macros/cgp_type.md)
/// generates for exactly this). An ordinary trait's associated type is fixed by whatever impl
/// supplies it, with no wiring entry to name, so it gets none.
fn assoc_mismatch_help(leaf: &Leaf) -> Option<String> {
    let Leaf::AssocTypeMismatch {
        owner,
        expected,
        expected_normalized,
        actual,
        component: Some(component),
        ..
    } = leaf
    else {
        return None;
    };
    // Unlike the header and the leaf, which *describe* the requirement and so lead with the
    // projection that names where it comes from, this `help` prescribes an edit the reader is meant
    // to type, so it takes the reduced form when there is one. `UseType<Tx<Postgres>>` is a wiring
    // entry; `UseType<Tx<<App as HasDbType>::Db>>` restates the requirement and leaves the reader to
    // reduce it.
    let required = expected_normalized.as_deref().unwrap_or(expected);
    Some(format!(
        "wire `{component}` to `UseType<{required}>` in the wiring for `{owner}`, or change the provider to work with `{actual}`"
    ))
}
