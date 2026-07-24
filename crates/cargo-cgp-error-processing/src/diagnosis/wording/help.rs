//! The `help` messages a resolved failure carries — the fixes its causes call for.

use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::resolved::Cause;

/// The distinct types that need a `#[derive(HasField)]`, in first-seen order — one per present
/// or `Deref`-reachable field (a `Deref`-reachable field points at its target, the type that
/// must actually derive). A genuinely missing field, or a non-field leaf, contributes none.
fn derive_targets(causes: &[Cause]) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    for cause in causes {
        let target = match &cause.leaf {
            Leaf::Field {
                owner,
                issue: FieldIssue::Present,
                ..
            } => owner,
            Leaf::Field {
                issue: FieldIssue::PresentViaDeref { target },
                ..
            } => target,
            Leaf::UnderivedFields { owner, .. } => owner,
            _ => continue,
        };
        if !targets.iter().any(|t| t == target) {
            targets.push(target.clone());
        }
    }
    targets
}

/// The `help` message per distinct type that must derive `HasField`, one per distinct derive
/// target of the resolved causes.
pub fn derive_help_messages(causes: &[Cause]) -> Vec<String> {
    derive_targets(causes)
        .into_iter()
        .map(|target| format!("make sure that `#[derive(HasField)]` is used for `{target}`"))
        .collect()
}

/// Every `help` a resolved failure's causes call for, in one list: the `#[derive(HasField)]` fixes
/// followed by the abstract-type wiring fixes. Both the streaming plan and the emitter's coalesced
/// block build their `help`s through this, so a block that merges several consumers carries the same
/// fixes as the per-consumer one it replaces.
pub fn fix_help_messages(causes: &[Cause]) -> Vec<String> {
    let mut helps = derive_help_messages(causes);
    helps.extend(assoc_mismatch_help_messages(causes));
    helps
}

/// The `help` message per abstract-type mismatch, naming the two ways to reconcile the two sides:
/// bind the component to the type the provider requires, or relax the provider to work with the one
/// the context supplies. Only a CGP abstract-type component earns a `help` — its concrete type is a
/// wiring choice, so there is a specific entry to change (`UseType<T>`, the provider
/// [`#[cgp_type]`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/cgp_type.md)
/// generates for exactly this). An ordinary trait's associated type is fixed by whatever impl
/// supplies it, with no wiring entry to name, so it contributes none. Emitted in first-seen order,
/// de-duplicated by component.
pub fn assoc_mismatch_help_messages(causes: &[Cause]) -> Vec<String> {
    let mut helps: Vec<String> = Vec::new();
    for cause in causes {
        let Leaf::AssocTypeMismatch {
            owner,
            expected,
            actual,
            component: Some(component),
            ..
        } = &cause.leaf
        else {
            continue;
        };
        let help = format!(
            "wire `{component}` to `UseType<{expected}>` in the wiring for `{owner}`, or change the provider to work with `{actual}`"
        );
        if !helps.contains(&help) {
            helps.push(help);
        }
    }
    helps
}
