//! Turning a resolved wiring failure into a plan for the emitter to apply.
//!
//! [`plan_resolved`] is the rustc-free heart of the typed-resolution transform: given the
//! recovered [`Resolved`], the diagnostic's kind and main-message text, and the component-name
//! map, it decides the rewritten main message (when the diagnostic is an identified CGP class)
//! and the replacement sub-messages (the derive `help`s and one `root cause:` note per cause).
//! The driver's emitter feeds it those inputs from the live `DiagInner` and turns the returned
//! [`DiagnosisPlan`] strings into `rustc` sub-diagnostics — so all the wording logic is here,
//! unit-tested without a compiler, and the emitter is left with only `DiagInner` manipulation.

use crate::diagnosis::coalesce::coalesce_underived_fields;
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::{
    cause_notes, consumer_header, derive_help_messages, field_mismatch_header, mismatch_leaf,
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
    /// The `help` messages naming each type that must derive `HasField`.
    pub helps: Vec<String>,
    /// The `root cause:` note — every cause's paths folded into one dependency graph. A single
    /// element, or none when there are no causes.
    pub notes: Vec<String>,
}

/// Build the [`DiagnosisPlan`] for a resolved failure. The main message is rewritten only when
/// it is an identified CGP class (see [`categorized_header`]); the sub-messages are replaced
/// either way. When the kept main message already states the leaf bound, the matching note drops
/// its `root cause:` lead so it does not repeat the header. Causes that share one fix — several
/// underived fields on one struct — are first coalesced into a single cause
/// ([`coalesce_underived_fields`]), so the note lists one root cause per required fix.
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

    let causes = coalesce_underived_fields(&resolved.causes);
    let helps = derive_help_messages(&causes);
    let notes = cause_notes(&causes, header_bound.as_deref());

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
    // A field-mismatch-coded (`E0271`) failure that the resolver traced to a *non*-mismatch cause —
    // a missing field or wiring reached through an opaque-future or associated-type projection, as a
    // manual `Send`-recovery wrapper's forwarding `async fn` produces — is a consequence of the
    // consumer failing, not a field-type mismatch. Its rustc message (`type mismatch resolving …`)
    // is opaque, so name the consumer trait that could not be implemented instead.
    if kind == DiagKind::FieldMismatch {
        return Some(consumer_header(resolved));
    }
    if let Some(text) = main_message {
        if let Some(parsed) = parse_trait_bound(text) {
            if parsed.trait_name == "CanUseComponent" {
                return Some(consumer_header(resolved));
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
                return Some(consumer_header(resolved));
            }
            // rustc opened the diagnostic on a bound that restates a genuine recovered leaf — an
            // ordinary bound such as `f64: Eq` the solver descended to *is* the root cause — so
            // keep rustc's header, which already names that cause. (The matching note then drops
            // its `root cause:` lead so it does not repeat the header.)
            if bound_is_leaf(resolved, parsed.bound) {
                return None;
            }
        }
        // A failure recovered at the use site reports the consumer trait the *call* needs. rustc's
        // own headline names whichever provider bound its solver stopped on — at a use site that is
        // usually dispatch plumbing (`PipeHandlers`, `ComposeHandlers`) the programmer never
        // asserted on, so the provider-form rewrite below would leak internals the call never
        // mentions. (A provider-side headline the programmer *did* assert — a `#[check_providers]`
        // layer — arrives as a `Check`, not a use-site kind, and keeps the provider form.)
        if kind == DiagKind::MethodNotFound {
            return Some(consumer_header(resolved));
        }
        if let Some(rewritten) = rewrite_trait_bound(text, names) {
            return Some(rewritten);
        }
        // The main message is a trait bound, but not a recognized CGP wiring bound and not a
        // recovered leaf: rustc descended to a mid-chain *symptom* (a getter bound on a request,
        // say, whose real cause is a missing wiring one level down). Naming the consumer trait the
        // context fails to implement is truer than leaking that symptom bound as the headline.
        if parse_trait_bound(text).is_some() {
            return Some(consumer_header(resolved));
        }
    }
    if kind == DiagKind::MethodNotFound {
        return Some(consumer_header(resolved));
    }
    None
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
