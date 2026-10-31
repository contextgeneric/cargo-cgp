//! The plan the emitter applies, and the function that builds it.

use crate::diagnosis::coalesce::coalesce_underived_fields;
use crate::diagnosis::plan::categorized_header::{
    Header, bound_leaf, categorized_header, split_code,
};
use crate::diagnosis::plan::{DiagKind, PendingNote};
use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::{cause_headline, fix_help_messages};
use crate::rewrite::{ComponentNameMap, parse_trait_bound};

/// The plan for transforming a resolved wiring failure's diagnostic: the replacement main
/// message (or `None` to keep rustc's own), the `help` sub-messages, and the `note` sub-messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosisPlan {
    /// The displayed main message: the `[CGP-Exxx]` code and the cause (see
    /// [`cause_headline`]), or `None` when the original is not an identified
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
    /// (see [`fix_help_messages`]).
    pub helps: Vec<String>,
    /// The `root cause:` note, **not yet rendered** — see [`PendingNote`].
    pub note: PendingNote,
}

impl DiagnosisPlan {
    /// Whether the block may join a coalesced block at flush: its headline was rewritten and does
    /// not state a single leaf of its own (a type mismatch, or a kept rustc headline over an
    /// ordinary bound), so the block reads the same when merged with others sharing its cause.
    pub fn coalesces(&self) -> bool {
        self.header.is_some() && self.note.header_leaf.is_none()
    }
}

/// Build the [`DiagnosisPlan`] for a resolved failure. The main message is rewritten only when
/// it is an identified CGP class (see `categorized_header`), and then it states the cause: a
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
