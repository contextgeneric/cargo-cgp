//! Choosing the fixes a cause's leaf calls for.

use crate::diagnosis::leaf::{
    Adapter, AdapterFix, CoreProviderFix, CoreProviderHint, DeriveFix, EntryFix, FieldIssue, Leaf,
    MethodFix, NearMiss, ProvidedComponent, ProvidesFix, SuggestedCall,
};
use crate::diagnosis::wording::fix_kind::FixKind;

/// The fixes a cause's leaf calls for, each as its kind, the owner it lands on, and the item it
/// names: a near miss first, since a slipped name is the likelier mistake, then the fix the cause
/// itself states. Empty for a cause whose fix the tool does not word.
pub fn fixes_for(leaf: &Leaf) -> Vec<(FixKind, String, Option<String>)> {
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
        Leaf::NotAProvider {
            provider,
            provider_trait,
            adapter: AdapterFix(adapters),
            provides: ProvidesFix(provides),
            core: CoreProviderFix(core),
        } => {
            fixes.extend(other_component_help(provider, provider_trait, provides));
            fixes.extend(adapter_help(provider, adapters));
            fixes.extend(core.as_ref().map(|hint| core_provider_help(provider, hint)));
        }
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

/// The `help` for a wired type an adapter makes a provider: the adapters to wire in its place, with
/// the `use` each needs when the prelude does not export it. `None` when no adapter is confirmed.
fn adapter_help(provider: &str, adapters: &[Adapter]) -> Option<(FixKind, String, Option<String>)> {
    if adapters.is_empty() {
        return None;
    }
    let named: Vec<String> = adapters
        .iter()
        .map(|adapter| format!("`{}`", adapter.ty))
        .collect();
    let imports: Vec<String> = adapters
        .iter()
        .filter_map(|adapter| adapter.import.as_ref())
        .map(|path| format!("`use {path};`"))
        .collect();
    let scope = if imports.is_empty() {
        String::new()
    } else {
        format!(", with {} in scope", imports.join(" and "))
    };
    Some(whole(format!(
        "wire {} in place of `{provider}`{scope}",
        named.join(" or ")
    )))
}

/// The `help` for a misused CGP core provider: what serves the slot in its place.
fn core_provider_help(
    provider: &str,
    hint: &CoreProviderHint,
) -> (FixKind, String, Option<String>) {
    match hint {
        CoreProviderHint::UseFields => whole(format!(
            "wire `UseFields` in place of `{provider}`, which reads each method's field by the method's name"
        )),
        CoreProviderHint::OpenDispatch { component, params } => whole(format!(
            "dispatch with `open {component};` and one `@{component}.{}: <provider>` entry per type, in place of `{provider}`",
            params.join(".")
        )),
    }
}

/// The `help` for a wired type that is a provider of other components, when it is not a provider of
/// the slot's own: it belongs under one of those components' keys, or a provider of this trait
/// belongs here.
fn other_component_help(
    provider: &str,
    provider_trait: &str,
    provides: &[ProvidedComponent],
) -> Option<(FixKind, String, Option<String>)> {
    if provides.is_empty() {
        return None;
    }
    let keys: Vec<String> = provides
        .iter()
        .map(|provided| format!("`{}`", provided.component))
        .collect();
    Some(whole(format!(
        "`{provider}` provides {}: wire it under {}, or wire a provider of `{provider_trait}` here",
        provides
            .iter()
            .map(|provided| format!("`{}`", provided.provider_trait))
            .collect::<Vec<_>>()
            .join(" and "),
        keys.join(" or ")
    )))
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
    // entry; `UseType<Tx<<App as HasDbType>::Db>>` restates the requirement and leaves the reader
    // to reduce it.
    let required = expected_normalized.as_deref().unwrap_or(expected);
    Some(format!(
        "wire `{component}` to `UseType<{required}>` in the wiring for `{owner}`, or change the provider to work with `{actual}`"
    ))
}
