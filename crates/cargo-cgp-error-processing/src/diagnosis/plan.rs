//! Turning a resolved wiring failure into a plan for the emitter to apply.
//!
//! [`plan_resolved`] is the rustc-free heart of the typed-resolution transform: given the
//! recovered [`Resolved`], the diagnostic's kind and main-message text, and the component-name
//! map, it decides the rewritten main message (when the diagnostic is an identified CGP class)
//! and the replacement sub-messages (the derive `help`s and one `root cause:` note per cause).
//! The driver's emitter feeds it those inputs from the live `DiagInner` and turns the returned
//! [`DiagnosisPlan`] strings into `rustc` sub-diagnostics — so all the wording logic is here,
//! unit-tested without a compiler, and the emitter is left with only `DiagInner` manipulation.

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::{
    cause_note, consumer_header, derive_help_messages, field_mismatch_header, mismatch_leaf,
};
use crate::rewrite::{ComponentNameMap, parse_trait_bound, rewrite_trait_bound};

/// The `rustc` error-code discriminant [`plan_resolved`] needs, in rustc-free form so the plan
/// is decided without linking the compiler. The emitter maps a diagnostic's `rustc` code to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagKind {
    /// A check-trait or ordinary-bound failure — rustc's `E0277`, the default.
    Check,
    /// A field-type mismatch — rustc's `E0271`, traced to a `HasField` projection.
    FieldMismatch,
    /// A consumer-method call failure — rustc's `E0599`, recovered at the use site.
    MethodNotFound,
}

/// The plan for transforming a resolved wiring failure's diagnostic: the replacement main
/// message (or `None` to keep rustc's own), the `help` sub-messages, and the `note` sub-messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosisPlan {
    /// The rewritten, `[CGP-Exxx]`-coded main message, or `None` when the original is not an
    /// identified CGP class and must be kept.
    pub header: Option<String>,
    /// The `help` messages naming each type that must derive `HasField`.
    pub helps: Vec<String>,
    /// One `note` message per recovered root cause, each carrying its dependency chain.
    pub notes: Vec<String>,
}

/// Build the [`DiagnosisPlan`] for a resolved failure. The main message is rewritten only when
/// it is an identified CGP class (see [`categorized_header`]); the sub-messages are replaced
/// either way. When the kept main message already states the leaf bound, the matching note drops
/// its `root cause:` lead so it does not repeat the header.
pub fn plan_resolved(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &Resolved,
    names: &ComponentNameMap,
) -> DiagnosisPlan {
    let header = categorized_header(kind, main_message, resolved, names);
    // The bound the kept main message states, if any, so a note does not restate it as its root
    // cause; a rewritten header makes it moot.
    let header_bound = if header.is_some() {
        None
    } else {
        main_message
            .and_then(parse_trait_bound)
            .map(|parsed| parsed.bound.to_owned())
    };

    let helps = derive_help_messages(&resolved.causes);
    let notes = resolved
        .causes
        .iter()
        .map(|cause| cause_note(cause, header_bound.as_deref()))
        .collect();

    DiagnosisPlan {
        header,
        helps,
        notes,
    }
}

/// The rewritten, `[CGP-Exxx]`-coded main message for a resolved failure — or `None` when the
/// original main message is not an identified CGP error class and must be kept (an ordinary
/// bound such as `f64: Eq` the solver already descended to). A field-type mismatch
/// ([`DiagKind::FieldMismatch`]) the resolver traced to a `HasField` projection becomes the
/// `[CGP-E003]` field form, worded from the mismatch leaf. An unsatisfied `CanUseComponent`
/// bound and a consumer-method [`DiagKind::MethodNotFound`] (whose text names no wiring trait)
/// are both worded from the typed resolution, whose full-path marker keys make the consumer name
/// exact; an unsatisfied `IsProviderFor` bound rewrites by its text, since the resolution does
/// not carry the provider-side names.
fn categorized_header(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &Resolved,
    names: &ComponentNameMap,
) -> Option<String> {
    // A field-type mismatch (`E0271`) the resolver traced to a `HasField` projection is its own
    // class, worded from the mismatch leaf rather than from the consumer trait.
    if kind == DiagKind::FieldMismatch
        && let Some(Leaf::FieldTypeMismatch {
            name,
            owner,
            expected,
            actual,
        }) = mismatch_leaf(resolved)
    {
        return Some(field_mismatch_header(name, owner, expected, actual));
    }
    if resolved.consumers.is_empty() {
        return None;
    }
    if let Some(text) = main_message {
        if let Some(parsed) = parse_trait_bound(text)
            && parsed.trait_name == "CanUseComponent"
        {
            return Some(consumer_header(resolved));
        }
        if let Some(rewritten) = rewrite_trait_bound(text, names) {
            return Some(rewritten);
        }
    }
    if kind == DiagKind::MethodNotFound {
        return Some(consumer_header(resolved));
    }
    None
}
