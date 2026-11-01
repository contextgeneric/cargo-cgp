//! The headline a resolved failure's class calls for: its type-mismatch or bound-text decision.

use crate::code::{ABSTRACT_TYPE_MISMATCH, FIELD_TYPE_MISMATCH};
use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::plan::DiagKind;
use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::{
    assoc_mismatch_leaf, class_code, consumer_header, mismatch_leaf, root_cause_lead,
    variant_mismatch_leaf,
};
use crate::rewrite::{ComponentNameMap, ParsedTraitBound, parse_trait_bound, rewrite_trait_bound};

/// What [`categorized_header`] decided: the main message to show, and the leaf it states.
pub struct Header<'a> {
    /// The rewritten main message, or `None` to keep rustc's own.
    pub text: Option<String>,
    /// The leaf the rewritten message states in full, when it was worded from one, so the note can
    /// drop that leaf's now-redundant `root cause:` lead.
    pub states_leaf: Option<&'a Leaf>,
    /// The headline the text rewrite gives rustc's own message, when it is a different class
    /// sentence a declined re-report of the same failure would carry.
    pub declined: Option<String>,
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
pub fn categorized_header<'a>(
    kind: DiagKind,
    main_message: Option<&str>,
    resolved: &'a Resolved,
    names: &ComponentNameMap,
) -> Header<'a> {
    if kind == DiagKind::TypeMismatch
        && let Some(header) = type_mismatch_header(resolved)
    {
        return header;
    }
    if resolved.consumers.is_empty() {
        return Header::keep();
    }
    // A mismatch-coded (`E0271`) failure that the resolver traced to a *non*-mismatch cause (a
    // missing field or wiring reached through an opaque-future or associated-type projection, as a
    // manual `Send`-recovery wrapper's forwarding `async fn` produces), is a consequence of the
    // consumer failing, not a type mismatch. Its rustc message (`type mismatch resolving …`) is
    // opaque, so name the consumer trait that could not be implemented instead.
    if kind == DiagKind::TypeMismatch {
        return Header::rewritten(consumer_header(resolved));
    }
    if let Some(text) = main_message
        && let Some(parsed) = parse_trait_bound(text)
    {
        return bound_text_header(resolved, text, &parsed, names);
    }
    // A failure recovered at the use site reports the consumer trait the *call* needs.
    if kind == DiagKind::MethodNotFound {
        return Header::rewritten(consumer_header(resolved));
    }
    Header::keep()
}

/// The headline of a projection mismatch stated by its own leaf, or `None` when the resolver traced it
/// to another cause.
pub fn type_mismatch_header(resolved: &Resolved) -> Option<Header<'_>> {
    // A projection mismatch (`E0271`) the resolver traced to a failing associated type is its own
    // class, worded from the mismatch leaf rather than from the consumer trait: the `[CGP-E004]`
    // field form for a `HasField` value, the `[CGP-E005]` abstract-type form for any other. The
    // field form is tried first, since a `HasField` value type is the more specific classification.
    if let Some(leaf) = mismatch_leaf(resolved) {
        return Some(Header {
            text: Some(format!("[{FIELD_TYPE_MISMATCH}] {}", root_cause_lead(leaf))),
            states_leaf: Some(leaf),
            declined: None,
        });
    }
    if let Some(leaf) = assoc_mismatch_leaf(resolved) {
        return Some(Header {
            text: Some(format!(
                "[{ABSTRACT_TYPE_MISMATCH}] {}",
                root_cause_lead(leaf)
            )),
            states_leaf: Some(leaf),
            declined: None,
        });
    }
    // A variant payload of the wrong type is an extensible-data cause, not a class of its own:
    // the block keeps its anchor's class code and states the mismatch, as the two classes
    // above do.
    if let Some(leaf) = variant_mismatch_leaf(resolved) {
        return Some(Header {
            text: Some(format!(
                "[{}] {}",
                class_code(resolved),
                root_cause_lead(leaf)
            )),
            states_leaf: Some(leaf),
            declined: None,
        });
    }
    None
}

/// The headline of a failure whose main message `text` is the trait bound `parsed`.
pub fn bound_text_header<'a>(
    resolved: &'a Resolved,
    text: &str,
    parsed: &ParsedTraitBound<'_>,
    names: &ComponentNameMap,
) -> Header<'a> {
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
    // rustc opened the diagnostic on a bound that restates a genuine recovered leaf: an
    // ordinary bound such as `f64: Eq` the solver descended to *is* the root cause, so
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
    Header::rewritten(consumer_header(resolved))
}

/// The recovered [`Leaf::Bound`] root cause that `bound` states, if any: `bound` is the whole
/// `Self: Trait<…>` restatement rustc's main message opened with. When there is one, rustc's header
/// already names the genuine root cause; when there is none, the header is a mid-chain symptom the
/// solver stopped on. The leaf writes a CGP empty list as `Product![]` or `Sum![]` where rustc
/// prints `Nil` or `Void`, the one difference between the two printings, so the comparison reads
/// the leaf back in rustc's form.
pub fn bound_leaf<'a>(resolved: &'a Resolved, bound: &str) -> Option<&'a Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::Bound { summary } if as_rustc_prints(summary) == bound))
}

/// A bound as rustc prints it: the leaf's `Product![]` and `Sum![]` back to `Nil` and `Void`.
pub fn as_rustc_prints(summary: &str) -> String {
    summary
        .replace("Product![]", "Nil")
        .replace("Sum![]", "Void")
}

/// Split a coded headline `[CGP-Exxx] sentence` into its code and its sentence.
pub fn split_code(header: &str) -> Option<(&str, &str)> {
    let rest = header.strip_prefix('[')?;
    let (code, sentence) = rest.split_once("] ")?;
    code.starts_with("CGP-E").then_some((code, sentence))
}
