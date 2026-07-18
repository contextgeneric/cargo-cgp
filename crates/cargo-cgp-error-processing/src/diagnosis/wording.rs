//! Wording a resolved check failure as diagnostic text.
//!
//! These are the pure `Resolved`-to-`String` builders the driver's emitter used to inline: the
//! coded main-message headers, the `root cause:` note bodies over their dependency chains, and
//! the `#[derive(HasField)]` help messages. They live here, apart from the driver, so each is a
//! plain function over the rustc-free [`Resolved`] model and is unit-tested without a compiler.
//! The [plan](super::plan) module composes them into the whole [`DiagnosisPlan`](super::DiagnosisPlan);
//! the emitter only turns that plan's strings into `rustc` sub-diagnostics.

use crate::code::{
    CONSUMER_TRAIT_UNIMPLEMENTED, DEP_FIELD_TYPE_MISMATCH, DEP_MISSING_DELEGATE_ENTRY,
    DEP_MISSING_FIELD, DEP_UNIMPLEMENTED_ACCESSOR, FIELD_TYPE_MISMATCH, ROOT_CAUSE_ORDINARY_BOUND,
    WRAPPER_TRAIT_UNIMPLEMENTED,
};
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

/// The main message for a resolved failure: the trait(s) the context fails to implement, taken
/// from the typed resolution — which keys each component marker by its full path — so two same-named
/// components in different modules can never be confused. A CGP consumer trait reads as `the consumer
/// trait …` (`CGP-E001`); a hand-written wrapper trait (not a CGP consumer) reads as `the trait …`
/// (`CGP-E009`), chosen by [`Resolved::consumers_are_cgp`]. The subject is named `context \`X\`` when
/// it is the checked context, or plainly `` `X` `` when it is a foreign wrapper type that merely
/// holds the context (`Router<Arc<MockApp>>`), chosen by [`Resolved::subject_is_context`].
pub fn consumer_header(resolved: &Resolved) -> String {
    let (noun, verb) = if resolved.consumers.len() == 1 {
        ("trait", "is")
    } else {
        ("traits", "are")
    };
    let list = quoted_list(&resolved.consumers);
    // A foreign wrapper subject (`subject_is_context == false`) is named plainly, so it is not
    // mislabelled a context.
    let subject = if resolved.subject_is_context {
        format!("context `{}`", resolved.context)
    } else {
        format!("`{}`", resolved.context)
    };
    if resolved.consumers_are_cgp {
        format!(
            "[{CONSUMER_TRAIT_UNIMPLEMENTED}] the consumer {noun} {list} {verb} not implemented for {subject}"
        )
    } else {
        format!(
            "[{WRAPPER_TRAIT_UNIMPLEMENTED}] the {noun} {list} {verb} not implemented for {subject}"
        )
    }
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

/// The one root-cause statement for a leaf — what the note names before the dependency chain,
/// and (repeated) the tree's terminal leaf the driver appends so the chain visibly bottoms out at
/// the root cause. A genuinely missing field is said plainly (without a `context` qualifier, since
/// `HasField` can land on any struct); a present-but-underived field is worded as the unimplemented
/// accessor, with the fix (the derive) carried by a separate `help`; a missing wiring (plain or a
/// redirect) names the missing delegate entry; any other leaf restates its unmet bound.
pub fn root_cause_lead(leaf: &Leaf) -> String {
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

/// A span-independent signature identifying *the same wiring failure* across the several
/// diagnostics one mistake produces — the check entry, each hand-written `impl` that references
/// the broken component, and each call site all recover the same [`Resolved`]. Two diagnostics
/// with an equal signature need the same fix at the same place, so the emitter shows the first and
/// suppresses the rest. It is built from the context, the failing consumer trait(s), and the
/// root-cause lead of each leaf — everything that identifies the failure and nothing tied to where
/// it surfaced — so two distinct broken endpoints (different consumers) or two different causes
/// keep distinct signatures and are each still reported.
pub fn cause_signature(resolved: &Resolved) -> String {
    let mut consumers = resolved.consumers.clone();
    consumers.sort();
    let mut leads: Vec<String> = resolved
        .causes
        .iter()
        .map(|cause| root_cause_lead(&cause.leaf))
        .collect();
    leads.sort();
    // `\u{1f}` (unit separator) cannot occur in a type or trait name, so joining on it makes the
    // signature unambiguous without escaping.
    format!(
        "{}\u{1f}{}\u{1f}{}",
        resolved.context,
        consumers.join("\u{1e}"),
        leads.join("\u{1e}"),
    )
}

/// The `CGP-E1xx` code for the terminal root-cause leaf as a dependency-tree entry, or `None` when
/// the leaf is a pass-through of a non-CGP bound (`the trait bound … is not satisfied`), which
/// carries no code. Keyed by leaf kind — a missing field, a present-but-underived field, a missing
/// delegate entry (plain or redirect), and a field-type mismatch each get their own code.
pub fn dependency_leaf_code(leaf: &Leaf) -> Option<&'static str> {
    match leaf {
        Leaf::Field {
            issue: FieldIssue::Missing,
            ..
        } => Some(DEP_MISSING_FIELD),
        Leaf::Field { .. } => Some(DEP_UNIMPLEMENTED_ACCESSOR),
        Leaf::MissingWiring { .. } | Leaf::MissingRedirectWiring { .. } => {
            Some(DEP_MISSING_DELEGATE_ENTRY)
        }
        Leaf::FieldTypeMismatch { .. } => Some(DEP_FIELD_TYPE_MISMATCH),
        Leaf::Bound { .. } => None,
    }
}

/// The terminal root-cause leaf as it appears *in* the dependency tree — [`root_cause_lead`] with
/// its `CGP-E1xx` code prefixed, or bare when the leaf is a pass-through non-CGP bound. This is the
/// coded counterpart the driver appends as the tree's last node; the `root cause:` note lead
/// ([`cause_note`]) repeats the same text with the [`root_cause_code`] tag.
pub fn dependency_tree_leaf(leaf: &Leaf) -> String {
    match dependency_leaf_code(leaf) {
        Some(code) => format!("[{code}] {}", root_cause_lead(leaf)),
        None => root_cause_lead(leaf),
    }
}

/// The `CGP-Exxx` code the `root cause:` note lead carries. It reuses the terminal leaf's
/// [`dependency_leaf_code`] where the leaf has one, so the lead and the tree's terminal show the
/// same code; when the leaf is an uncoded pass-through bound it takes the `CGP-E2xx` root-cause code
/// [`ROOT_CAUSE_ORDINARY_BOUND`] instead, since the lead still names a classified root cause.
pub fn root_cause_code(leaf: &Leaf) -> &'static str {
    dependency_leaf_code(leaf).unwrap_or(ROOT_CAUSE_ORDINARY_BOUND)
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
        "root cause: [{code}] {lead}\nthis is required through the dependency chain:\n{indented}",
        code = root_cause_code(&cause.leaf),
        lead = root_cause_lead(&cause.leaf),
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
