//! The `#[derive(HasField)]` help messages a resolved failure carries.

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
