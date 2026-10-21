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

use crate::code::{ABSTRACT_TYPE_MISMATCH, FIELD_TYPE_MISMATCH};
use crate::diagnosis::coalesce::coalesce_underived_fields;
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::node::ChainNode;
use crate::diagnosis::resolved::{Cause, Resolved};
use crate::diagnosis::wording::{
    assoc_mismatch_leaf, cause_headline, cause_notes_seen, class_code, consumer_header,
    fix_help_messages, mismatch_leaf, root_cause_lead, variant_mismatch_leaf,
};
use crate::rewrite::{ComponentNameMap, parse_trait_bound, rewrite_trait_bound};

/// The `rustc` error-code discriminant [`plan_resolved`] needs, in rustc-free form so the plan
/// is decided without linking the compiler. The emitter maps a diagnostic's `rustc` code to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagKind {
    /// A check-trait or ordinary-bound failure — rustc's `E0277`, the default.
    Check,
    /// An associated-type mismatch — rustc's `E0271`, traced to a failing projection: a `HasField`
    /// value type (the `[CGP-E004]` field form) or any other associated type, most often a CGP
    /// abstract type (the `[CGP-E005]` form).
    TypeMismatch,
    /// A consumer-method call failure recovered at the use site — rustc's `E0599`, or an `E0277`
    /// whose obligation the resolver re-read from the call expression itself.
    MethodNotFound,
}

/// The plan for transforming a resolved wiring failure's diagnostic: the replacement main
/// message (or `None` to keep rustc's own), the `help` sub-messages, and the `note` sub-messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosisPlan {
    /// The displayed main message: the `[CGP-Exxx]` code and the cause (see
    /// [`cause_headline`](super::cause_headline)), or `None` when the original is not an identified
    /// CGP class and must be kept.
    pub header: Option<String>,
    /// The internal headline: the code and the class sentence, as a declined re-report of the same
    /// failure would be worded by the text rewrite. The de-duplication ledger matches on it; it is
    /// shown only when the causes are too mixed to state at once. `None` exactly when `header` is.
    pub class_header: Option<String>,
    /// The headline a declined re-report of the same failure would carry when it differs from the
    /// class headline: the provider form the text rewrite gives an `IsProviderFor` headline the
    /// resolver classed by its consumer. The ledger records it as a second key.
    pub declined_header: Option<String>,
    /// The `help` messages naming each fix the causes call for, in the order the causes are listed
    /// (see [`fix_help_messages`](super::fix_help_messages)).
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

impl DiagnosisPlan {
    /// Whether the block may join a coalesced block at flush: its headline was rewritten and does
    /// not state a single leaf of its own (a type mismatch, or a kept rustc headline over an
    /// ordinary bound), so the block reads the same when merged with others sharing its cause.
    pub fn coalesces(&self) -> bool {
        self.header.is_some() && self.note.header_leaf.is_none()
    }
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
/// it is an identified CGP class (see [`categorized_header`]), and then it states the cause: a
/// mismatch headline is its cause sentence, and every other class gives way to
/// [`cause_headline`], keeping the class sentence as the internal headline. The sub-messages are
/// replaced either way. When the main message already states the leaf (a mismatch, or a kept
/// rustc headline restating the ordinary bound), the note drops its `root cause:` lead so it does
/// not repeat the header. Causes that share one fix (several underived fields on one struct) are
/// first coalesced into a single cause ([`coalesce_underived_fields`]), so the note lists one root
/// cause per required fix. The note is returned unrendered, as a [`PendingNote`]; the caller renders
/// it once it knows what earlier notes have drawn.
pub fn plan_resolved(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &Resolved,
    names: &ComponentNameMap,
) -> DiagnosisPlan {
    let Header {
        text,
        states_leaf,
        declined,
    } = categorized_header(kind, main_message, resolved, names);
    // The leaf the main message already states, whether because the header was *rewritten* from
    // that leaf or because rustc's kept header restates the ordinary bound the walk descended to.
    // Its note then drops the `root cause:` lead rather than repeating the header.
    let header_leaf = states_leaf.or_else(|| {
        if text.is_some() {
            return None;
        }
        bound_leaf(resolved, parse_trait_bound(main_message?)?.bound)
    });

    let causes = coalesce_underived_fields(&resolved.causes);
    let helps = fix_help_messages(&causes);
    // A mismatch headline already is its cause; every other class headline gives way to the cause.
    let header = text.as_deref().map(|class| match states_leaf {
        Some(_) => class.to_owned(),
        None => match split_code(class) {
            Some((code, sentence)) => cause_headline(code, sentence, &causes),
            None => class.to_owned(),
        },
    });

    DiagnosisPlan {
        header,
        class_header: text,
        declined_header: declined,
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
    /// The headline the text rewrite gives rustc's own message, when it is a different class
    /// sentence a declined re-report of the same failure would carry.
    declined: Option<String>,
}

impl Header<'_> {
    /// A header that keeps rustc's own main message, stating no leaf of its own.
    fn keep() -> Self {
        Header {
            text: None,
            states_leaf: None,
            declined: None,
        }
    }

    /// A rewritten header worded from something other than a single leaf (a consumer trait, a
    /// provider bound), so no lead is redundant.
    fn rewritten(text: String) -> Self {
        Header {
            text: Some(text),
            states_leaf: None,
            declined: None,
        }
    }
}

/// The class headline for a resolved failure, coded `[CGP-Exxx]`, or `None` when the original
/// main message is not an identified CGP error class and must be kept (an ordinary bound such as
/// `f64: Eq` the solver already descended to). A projection mismatch ([`DiagKind::TypeMismatch`])
/// is the `[CGP-E004]` field class when the resolver traced it to a `HasField` projection, or the
/// `[CGP-E005]` abstract-type class for any other associated type, each worded as its mismatch
/// cause and reported as the leaf it states, so its note drops the now redundant lead. Every other
/// class is worded from the typed resolution, whose anchor decides it: the provider class for a
/// `#[check_providers]` entry or a provider's own method called directly, the consumer class for a
/// CGP consumer trait, and the plain-trait class for any other trait, whatever bound rustc's own
/// headline stopped on.
fn categorized_header<'a>(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &'a Resolved,
    names: &ComponentNameMap,
) -> Header<'a> {
    // A projection mismatch (`E0271`) the resolver traced to a failing associated type is its own
    // class, worded from the mismatch leaf rather than from the consumer trait: the `[CGP-E004]`
    // field form for a `HasField` value, the `[CGP-E005]` abstract-type form for any other. The
    // field form is tried first, since a `HasField` value type is the more specific classification.
    if kind == DiagKind::TypeMismatch {
        if let Some(leaf) = mismatch_leaf(resolved) {
            return Header {
                text: Some(format!("[{FIELD_TYPE_MISMATCH}] {}", root_cause_lead(leaf))),
                states_leaf: Some(leaf),
                declined: None,
            };
        }
        if let Some(leaf) = assoc_mismatch_leaf(resolved) {
            return Header {
                text: Some(format!(
                    "[{ABSTRACT_TYPE_MISMATCH}] {}",
                    root_cause_lead(leaf)
                )),
                states_leaf: Some(leaf),
                declined: None,
            };
        }
        // A variant payload of the wrong type is an extensible-data cause, not a class of its own:
        // the block keeps its anchor's class code and states the mismatch, as the two classes
        // above do.
        if let Some(leaf) = variant_mismatch_leaf(resolved) {
            return Header {
                text: Some(format!(
                    "[{}] {}",
                    class_code(resolved),
                    root_cause_lead(leaf)
                )),
                states_leaf: Some(leaf),
                declined: None,
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
    if let Some(text) = main_message
        && let Some(parsed) = parse_trait_bound(text)
    {
        if parsed.trait_name == "CanUseComponent" {
            return Header::rewritten(consumer_header(resolved));
        }
        // The class follows the anchor, never the provider rustc's bound happened to stop on:
        // that may be dispatch plumbing, a `RedirectLookup`, or a provider deep in the chain,
        // none of which the programmer asserted on. Only a `#[check_providers]` entry or a call
        // to a provider's own method, whose anchor records its provider, is the provider class.
        // The provider form the text rewrite gives rustc's own headline is kept as a second
        // key, since a declined re-report of the same failure is worded with it.
        if parsed.trait_name == "IsProviderFor" {
            return Header {
                declined: rewrite_trait_bound(text, names),
                ..Header::rewritten(consumer_header(resolved))
            };
        }
        // rustc opened the diagnostic on a bound that restates a genuine recovered leaf — an
        // ordinary bound such as `f64: Eq` the solver descended to *is* the root cause — so
        // keep rustc's header, which already names that cause. (The matching note then drops
        // its `root cause:` lead so it does not repeat the header.) Where the leaf writes the
        // bound as the program did and rustc does not (`Product![]` for CGP's `Nil`), the
        // header takes the leaf's wording in rustc's sentence instead.
        if let Some(leaf) = bound_leaf(resolved, parsed.bound) {
            return match leaf {
                Leaf::Bound { summary } if summary != parsed.bound => Header {
                    text: Some(root_cause_lead(leaf)),
                    states_leaf: Some(leaf),
                    declined: None,
                },
                _ => Header::keep(),
            };
        }
        // The main message is a trait bound, but not a recognized CGP wiring bound and not a
        // recovered leaf: rustc descended to a mid-chain *symptom* (a getter bound on a
        // request, say, whose real cause is a missing wiring one level down). Naming the
        // consumer trait the context fails to implement is truer than leaking that symptom
        // bound as the headline.
        return Header::rewritten(consumer_header(resolved));
    }
    // A failure recovered at the use site reports the consumer trait the *call* needs.
    if kind == DiagKind::MethodNotFound {
        return Header::rewritten(consumer_header(resolved));
    }
    Header::keep()
}

/// The recovered [`Leaf::Bound`] root cause that `bound` states, if any: `bound` is the whole
/// `Self: Trait<…>` restatement rustc's main message opened with. When there is one, rustc's header
/// already names the genuine root cause; when there is none, the header is a mid-chain symptom the
/// solver stopped on. The leaf writes a CGP empty list as `Product![]` or `Sum![]` where rustc
/// prints `Nil` or `Void`, the one difference between the two printings, so the comparison reads
/// the leaf back in rustc's form.
fn bound_leaf<'a>(resolved: &'a Resolved, bound: &str) -> Option<&'a Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::Bound { summary } if as_rustc_prints(summary) == bound))
}

/// A bound as rustc prints it: the leaf's `Product![]` and `Sum![]` back to `Nil` and `Void`.
fn as_rustc_prints(summary: &str) -> String {
    summary
        .replace("Product![]", "Nil")
        .replace("Sum![]", "Void")
}

/// Split a coded headline `[CGP-Exxx] sentence` into its code and its sentence.
fn split_code(header: &str) -> Option<(&str, &str)> {
    let rest = header.strip_prefix('[')?;
    let (code, sentence) = rest.split_once("] ")?;
    code.starts_with("CGP-E").then_some((code, sentence))
}
