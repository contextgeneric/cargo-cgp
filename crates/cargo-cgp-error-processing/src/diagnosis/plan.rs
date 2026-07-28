//! Turning a resolved wiring failure into a plan for the emitter to apply.
//!
//! [`plan_resolved`] is the rustc-free heart of the typed-resolution transform: given the
//! recovered [`Resolved`], the diagnostic's kind and main-message text, and the component-name
//! map, it decides the rewritten main message (when the diagnostic is an identified CGP class)
//! and the replacement sub-messages (the fix `help`s and the `root cause:` note).
//! The driver's emitter feeds it those inputs from the live `DiagInner` and turns the returned
//! [`DiagnosisPlan`] strings into `rustc` sub-diagnostics — so all the wording logic is here,
//! unit-tested without a compiler, and the emitter is left with only `DiagInner` manipulation.

use std::collections::HashSet;

use crate::diagnosis::coalesce::coalesce_underived_fields;
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::node::ChainNode;
use crate::diagnosis::resolved::{Cause, Resolved};
use crate::diagnosis::wording::{
    assoc_mismatch_header, assoc_mismatch_leaf, cause_notes_seen, consumer_header,
    field_mismatch_header, fix_help_messages, mismatch_leaf,
};
use crate::rewrite::{ComponentNameMap, parse_trait_bound, rewrite_trait_bound};

/// The `rustc` error-code discriminant [`plan_resolved`] needs, in rustc-free form so the plan
/// is decided without linking the compiler. The emitter maps a diagnostic's `rustc` code to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagKind {
    /// A check-trait or ordinary-bound failure — rustc's `E0277`, the default.
    Check,
    /// An associated-type mismatch — rustc's `E0271`, traced to a failing projection: a `HasField`
    /// value type (the `[CGP-E003]` field form) or any other associated type, most often a CGP
    /// abstract type (the `[CGP-E017]` form).
    TypeMismatch,
    /// A consumer-method call failure recovered at the use site — rustc's `E0599`, or an `E0277`
    /// whose obligation the resolver re-read from the call expression itself.
    MethodNotFound,
}

/// The plan for transforming a resolved wiring failure's diagnostic: the replacement main
/// message (or `None` to keep rustc's own), the `help` sub-messages, and the `note` sub-messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosisPlan {
    /// The rewritten, `[CGP-Exxx]`-coded main message, or `None` when the original is not an
    /// identified CGP class and must be kept.
    pub header: Option<String>,
    /// The `help` messages naming each fix the causes call for — a type that must derive
    /// `HasField`, or an abstract type whose wiring must change (see
    /// [`fix_help_messages`](super::fix_help_messages)).
    pub helps: Vec<String>,
    /// The `root cause:` note, **not yet rendered** — see [`PendingNote`].
    pub note: PendingNote,
}

/// The `root cause:` note as the inputs it renders from, rather than as text.
///
/// Rendering is deferred because a note elides against what *other* notes of the same compilation
/// already drew (see [`DependencyGraph::render_seen`](crate::DependencyGraph::render_seen)), and only
/// the emitter's flush knows the order they will appear in. Keeping the note in this form rather than
/// as a rendered string alongside is deliberate: two representations of one note would let the tests
/// pin one while the emitter shows the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingNote {
    /// The coalesced causes, one per distinct root cause.
    pub causes: Vec<Cause>,
    /// The leaf the header states, if any, whose lead is then redundant.
    pub header_leaf: Option<Leaf>,
}

impl PendingNote {
    /// Render the note against `seen`, the subtrees the compilation's earlier notes already drew. A
    /// caller with no such context — a unit test, or any single-diagnostic use — passes a fresh set.
    /// Yields one note, or none when there are no causes.
    pub fn render(&self, seen: &mut HashSet<ChainNode>) -> Vec<String> {
        cause_notes_seen(&self.causes, self.header_leaf.as_ref(), seen)
    }
}

/// Build the [`DiagnosisPlan`] for a resolved failure. The main message is rewritten only when
/// it is an identified CGP class (see [`categorized_header`]); the sub-messages are replaced
/// either way. When the kept main message already states the leaf bound, the matching note drops
/// its `root cause:` lead so it does not repeat the header. Causes that share one fix — several
/// underived fields on one struct — are first coalesced into a single cause
/// ([`coalesce_underived_fields`]), so the note lists one root cause per required fix. The note is
/// returned unrendered, as a [`PendingNote`]; the caller renders it once it knows what earlier notes
/// have drawn.
pub fn plan_resolved(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &Resolved,
    names: &ComponentNameMap,
) -> DiagnosisPlan {
    let Header { text, states_leaf } = categorized_header(kind, main_message, resolved, names);
    // The leaf the main message already states, whether because the header was *rewritten* from
    // that leaf or because rustc's kept header restates the ordinary bound the walk descended to.
    // Its note then drops the `root cause:` lead rather than repeating the header.
    let header_leaf = states_leaf.or_else(|| {
        if text.is_some() {
            return None;
        }
        let bound = parse_trait_bound(main_message?)?.bound;
        resolved
            .causes
            .iter()
            .map(|cause| &cause.leaf)
            .find(|leaf| matches!(leaf, Leaf::Bound { summary } if summary == bound))
    });

    let causes = coalesce_underived_fields(&resolved.causes);
    let helps = fix_help_messages(&causes);

    DiagnosisPlan {
        header: text,
        helps,
        note: PendingNote {
            header_leaf: header_leaf.cloned(),
            causes,
        },
    }
}

/// What [`categorized_header`] decided: the main message to show, and the leaf it states.
struct Header<'a> {
    /// The rewritten main message, or `None` to keep rustc's own.
    text: Option<String>,
    /// The leaf the rewritten message states in full, when it was worded from one — so the note can
    /// drop that leaf's now-redundant `root cause:` lead.
    states_leaf: Option<&'a Leaf>,
}

impl Header<'_> {
    /// A header that keeps rustc's own main message, stating no leaf of its own.
    fn keep() -> Self {
        Header {
            text: None,
            states_leaf: None,
        }
    }

    /// A rewritten header worded from something other than a single leaf (a consumer trait, a
    /// provider bound), so no lead is redundant.
    fn rewritten(text: String) -> Self {
        Header {
            text: Some(text),
            states_leaf: None,
        }
    }
}

/// The rewritten, `[CGP-Exxx]`-coded main message for a resolved failure — or `None` when the
/// original main message is not an identified CGP error class and must be kept (an ordinary
/// bound such as `f64: Eq` the solver already descended to). A projection mismatch
/// ([`DiagKind::TypeMismatch`]) becomes the `[CGP-E003]` field form when the resolver traced it to a
/// `HasField` projection, or the `[CGP-E017]` abstract-type form for any other associated type, each
/// worded from its mismatch leaf — and reported as the leaf it states, so its note drops the now
/// redundant lead. An unsatisfied `CanUseComponent`
/// bound and a consumer-method [`DiagKind::MethodNotFound`] (whose text names no wiring trait)
/// are both worded from the typed resolution, whose full-path marker keys make the consumer name
/// exact; an unsatisfied `IsProviderFor` bound rewrites by its text, since the resolution does
/// not carry the provider-side names.
fn categorized_header<'a>(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &'a Resolved,
    names: &ComponentNameMap,
) -> Header<'a> {
    // A projection mismatch (`E0271`) the resolver traced to a failing associated type is its own
    // class, worded from the mismatch leaf rather than from the consumer trait: the `[CGP-E003]`
    // field form for a `HasField` value, the `[CGP-E017]` abstract-type form for any other. The
    // field form is tried first, since a `HasField` value type is the more specific classification.
    if kind == DiagKind::TypeMismatch {
        if let Some(Leaf::FieldTypeMismatch {
            name,
            owner,
            expected,
            expected_normalized,
            actual,
        }) = mismatch_leaf(resolved)
        {
            return Header {
                text: Some(field_mismatch_header(
                    name,
                    owner,
                    expected,
                    expected_normalized.as_deref(),
                    actual,
                )),
                states_leaf: mismatch_leaf(resolved),
            };
        }
        if let Some(Leaf::AssocTypeMismatch {
            assoc,
            trait_name,
            owner,
            expected,
            expected_normalized,
            actual,
            component,
        }) = assoc_mismatch_leaf(resolved)
        {
            return Header {
                text: Some(assoc_mismatch_header(
                    assoc,
                    trait_name,
                    owner,
                    expected,
                    expected_normalized.as_deref(),
                    actual,
                    component.as_deref(),
                )),
                states_leaf: assoc_mismatch_leaf(resolved),
            };
        }
    }
    if resolved.consumers.is_empty() {
        return Header::keep();
    }
    // A mismatch-coded (`E0271`) failure that the resolver traced to a *non*-mismatch cause — a
    // missing field or wiring reached through an opaque-future or associated-type projection, as a
    // manual `Send`-recovery wrapper's forwarding `async fn` produces — is a consequence of the
    // consumer failing, not a type mismatch. Its rustc message (`type mismatch resolving …`) is
    // opaque, so name the consumer trait that could not be implemented instead.
    if kind == DiagKind::TypeMismatch {
        return Header::rewritten(consumer_header(resolved));
    }
    if let Some(text) = main_message {
        if let Some(parsed) = parse_trait_bound(text) {
            if parsed.trait_name == "CanUseComponent" {
                return Header::rewritten(consumer_header(resolved));
            }
            // An `IsProviderFor` bound whose subject is a `RedirectLookup` names only redirect
            // plumbing — the lookup resolved to *no* provider at all (the wiring is missing), so
            // there is no real provider to report. Naming `RedirectLookup<Ctx, @Path>` as the
            // "provider" leaks a type the programmer never wrote and stops at the redirect rather
            // than following through it. The resolution already recovered the consumer the redirect
            // stands for and the missing-wiring cause beneath it, so word the header from that
            // consumer instead — the same `[CGP-E001]` form the use-site path uses. (A real wired
            // provider whose dependency fails — `SerializeIterator`, say — keeps the provider form
            // below, since it names something the programmer chose.)
            if parsed.trait_name == "IsProviderFor" && subject_is_redirect_lookup(parsed.subject) {
                return Header::rewritten(consumer_header(resolved));
            }
            // rustc opened the diagnostic on a bound that restates a genuine recovered leaf — an
            // ordinary bound such as `f64: Eq` the solver descended to *is* the root cause — so
            // keep rustc's header, which already names that cause. (The matching note then drops
            // its `root cause:` lead so it does not repeat the header.)
            if bound_is_leaf(resolved, parsed.bound) {
                return Header::keep();
            }
        }
        // A failure recovered at the use site reports the consumer trait the *call* needs. rustc's
        // own headline names whichever provider bound its solver stopped on — at a use site that is
        // usually dispatch plumbing (`PipeHandlers`, `ComposeHandlers`) the programmer never
        // asserted on, so the provider-form rewrite below would leak internals the call never
        // mentions. (A provider-side headline the programmer *did* assert — a `#[check_providers]`
        // layer — arrives as a `Check`, not a use-site kind, and keeps the provider form.)
        if kind == DiagKind::MethodNotFound {
            return Header::rewritten(consumer_header(resolved));
        }
        if let Some(rewritten) = rewrite_trait_bound(text, names) {
            return Header::rewritten(rewritten);
        }
        // The main message is a trait bound, but not a recognized CGP wiring bound and not a
        // recovered leaf: rustc descended to a mid-chain *symptom* (a getter bound on a request,
        // say, whose real cause is a missing wiring one level down). Naming the consumer trait the
        // context fails to implement is truer than leaking that symptom bound as the headline.
        if parse_trait_bound(text).is_some() {
            return Header::rewritten(consumer_header(resolved));
        }
    }
    if kind == DiagKind::MethodNotFound {
        return Header::rewritten(consumer_header(resolved));
    }
    Header::keep()
}

/// Whether a trait bound's subject (self type) is CGP's `RedirectLookup` provider — the redirect
/// plumbing an `open`/namespace lookup routes through. Strips a leading `for<…>` higher-ranked
/// binder (a higher-ranked obligation prints as `for<'a> RedirectLookup<…>`), then the type's own
/// generic arguments and any module path, so it matches on the bare head segment.
fn subject_is_redirect_lookup(subject: &str) -> bool {
    let subject = subject.trim();
    // Drop a leading `for<'a, …>` binder; its `>` is the first one, since a lifetime list carries
    // no nested `<`.
    let ty = subject
        .strip_prefix("for<")
        .and_then(|rest| rest.split_once('>'))
        .map(|(_, rest)| rest.trim())
        .unwrap_or(subject);
    ty.split('<')
        .next()
        .unwrap_or("")
        .rsplit("::")
        .next()
        .unwrap_or("")
        .trim()
        == "RedirectLookup"
}

/// Whether `bound` — the whole `Self: Trait<…>` restatement rustc's main message opened with —
/// matches a recovered [`Leaf::Bound`] root cause. When it does, rustc's header already names the
/// genuine root cause and should be kept; when it does not, the header is a mid-chain symptom the
/// solver stopped on. Compared against the same [`Leaf::Bound::summary`] the note wording uses, so
/// the two stay in step.
fn bound_is_leaf(resolved: &Resolved, bound: &str) -> bool {
    resolved
        .causes
        .iter()
        .any(|cause| matches!(&cause.leaf, Leaf::Bound { summary } if summary == bound))
}
