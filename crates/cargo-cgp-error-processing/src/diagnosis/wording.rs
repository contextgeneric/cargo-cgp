//! Wording a resolved check failure as diagnostic text.
//!
//! These are the pure `Resolved`-to-`String` builders the driver's emitter used to inline: the
//! coded main-message headers, the `root cause:` note bodies over their dependency chains, and
//! the `#[derive(HasField)]` help messages. They live here, apart from the driver, so each is a
//! plain function over the rustc-free [`Resolved`] model and is unit-tested without a compiler.
//! The [plan](super::plan) module composes them into the whole [`DiagnosisPlan`](super::DiagnosisPlan);
//! the emitter only turns that plan's strings into `rustc` sub-diagnostics.

use crate::code::{CONSUMER_TRAIT_UNIMPLEMENTED, FIELD_TYPE_MISMATCH};
use crate::diagnosis::leaf::{FieldIssue, Leaf};
use crate::diagnosis::resolved::{Cause, Resolved};
use crate::tree::render_dependency_tree;

/// A `and`-joined, back-quoted list: `` `x` ``, `` `x` and `y` ``, or `` `x`, `y`, and `z` ``.
pub fn quoted_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("`{item}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The `[CGP-E001]` main message for a resolved failure: the consumer trait(s) the context
/// fails to implement, taken from the typed resolution — which keys each component marker by
/// its full path — so two same-named components in different modules can never be confused.
pub fn consumer_header(resolved: &Resolved) -> String {
    let (noun, verb) = if resolved.consumers.len() == 1 {
        ("trait", "is")
    } else {
        ("traits", "are")
    };
    format!(
        "[{CONSUMER_TRAIT_UNIMPLEMENTED}] the consumer {noun} {list} {verb} not implemented for context `{context}`",
        list = quoted_list(&resolved.consumers),
        context = resolved.context,
    )
}

/// The `[CGP-E003]` main message for a field-type mismatch: the field the wiring reads carries
/// the wrong type. The expected type comes from the failing projection and the actual from
/// querying the struct by `DefId`, so the two are read from the real type, never a same-named
/// one elsewhere.
pub fn field_mismatch_header(name: &str, owner: &str, expected: &str, actual: &str) -> String {
    format!(
        "[{FIELD_TYPE_MISMATCH}] expected a `{name}` field of type `{expected}` on `{owner}`, but found `{actual}`"
    )
}

/// The first field-type-mismatch cause of a resolution, if any — the leaf `[CGP-E003]` is
/// worded from.
pub fn mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::FieldTypeMismatch { .. }))
}

/// The statement that a context carries no delegate entry for a key — a component marker for a
/// plain missing wiring, or a redirect path for an unterminated namespace lookup. Shared by the
/// root-cause lead here and the terminal dependency-tree node the driver renders, so the two never
/// drift: a missing delegate entry reads the same whether it heads the note or ends the chain.
pub fn missing_delegate_entry(context: &str, key: &str) -> String {
    format!("context `{context}` does not contain any delegate entry for `{key}`")
}

/// The one root-cause lead line for a leaf — what the note names before the dependency chain.
/// A genuinely missing field is said plainly (without a `context` qualifier, since `HasField`
/// can land on any struct); a present-but-underived field is worded as the unimplemented
/// accessor, with the fix (the derive) carried by a separate `help`; a component the context
/// does not wire names the missing component; any other leaf restates its unmet bound.
fn root_cause_lead(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
        } => format!("missing field `{name}` on `{owner}`"),
        Leaf::MissingWiring { component, owner } => missing_delegate_entry(owner, component),
        Leaf::MissingRedirectWiring { path, context } => missing_delegate_entry(context, path),
        Leaf::Field { name, owner, .. } => {
            format!(
                "accessor trait `HasField` with field `{name}` is not implemented for `{owner}`"
            )
        }
        Leaf::FieldTypeMismatch {
            name,
            owner,
            expected,
            actual,
        } => {
            format!("field `{name}` on `{owner}` has type `{actual}`, but `{expected}` is required")
        }
        Leaf::Bound { summary } => format!("the trait bound `{summary}` is not satisfied"),
    }
}

/// The note body for one root cause: the `root cause:` lead naming the leaf, then the rendered
/// dependency chain nested beneath its heading. When the diagnostic's kept main message already
/// states the leaf bound (`header_bound`), the lead would only repeat it, so the note carries
/// the chain alone — as it also does for a field-type mismatch, whose `[CGP-E003]` header
/// states the leaf in full.
pub fn cause_note(cause: &Cause, header_bound: Option<&str>) -> String {
    let chain = render_dependency_tree(&cause.tree);
    if let Leaf::FieldTypeMismatch { .. } = &cause.leaf {
        return format!("this is required through the dependency chain:\n{chain}");
    }
    if let (Some(bound), Leaf::Bound { summary }) = (header_bound, &cause.leaf)
        && summary == bound
    {
        return format!("this is required through the dependency chain:\n{chain}");
    }
    let indented: String = chain
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "root cause: {}\nthis is required through the dependency chain:\n{indented}",
        root_cause_lead(&cause.leaf),
    )
}

/// The distinct types that need a `#[derive(HasField)]`, in first-seen order — one per present
/// or `Deref`-reachable field (a `Deref`-reachable field points at its target, the type that
/// must actually derive). A genuinely missing field, or a non-field leaf, contributes none.
fn derive_targets(causes: &[Cause]) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    for cause in causes {
        let Leaf::Field { owner, issue, .. } = &cause.leaf else {
            continue;
        };
        let target = match issue {
            FieldIssue::Missing => continue,
            FieldIssue::Present => owner,
            FieldIssue::PresentViaDeref { target } => target,
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
