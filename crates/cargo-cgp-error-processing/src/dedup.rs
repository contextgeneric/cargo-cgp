//! Span-independent de-duplication of the tool's transformed diagnostics.
//!
//! CGP wiring is lazy, so one mistake surfaces the same error at many sites — the
//! `check_components!` entry, every hand-written `impl` that references the broken component, and
//! each call. The emitter records each transformed diagnostic here and suppresses a later one whose
//! signature it has already seen, so a mistake is shown once. Every key is span-independent, since
//! the span is exactly what differs between the copies. Only the tool's own transformed diagnostics
//! are recorded; an untouched `rustc` error never reaches this ledger.

use std::collections::HashSet;

/// The ledger of CGP diagnostics already emitted this compilation, keyed by span-independent
/// signature.
#[derive(Debug, Default)]
pub struct DedupLedger {
    seen: HashSet<String>,
}

impl DedupLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one transformed diagnostic and report whether an equal one was already recorded —
    /// `true` means this diagnostic is a re-report the caller should suppress.
    ///
    /// A resolved diagnostic is keyed by its recovered `cause_signature`
    /// ([`cause_signature`](crate::cause_signature)); a declined-but-rewritten one by
    /// `text_signature`, its rendered message text — computed lazily, since it is only needed when
    /// no cause was recovered. A second key is the **coded main-message header** (a `[CGP-E0xx]`
    /// main message), and it works in one direction only: a failure the resolver declined but still
    /// rewrote falls back to raw `IsProviderFor` scaffolding, yet carries the *same* coded header
    /// as the resolved tree of the same failure, so a declined report is suppressed when a resolved
    /// one already recorded its header. A resolved report never consults the header, so a declined
    /// fallback that arrived first never hides the resolved tree, and two resolved reports of one
    /// consumer with different causes are both kept (the flush coalesces what they share). The
    /// header key is restricted to a `[CGP-E0` prefix (a main-message code), so a kept rustc header
    /// is never a de-duplication key.
    pub fn check_and_record(
        &mut self,
        cause_signature: Option<&str>,
        text_signature: impl FnOnce() -> String,
        main_message: Option<&str>,
    ) -> bool {
        let header = main_message
            .filter(|header| header.starts_with("[CGP-E0"))
            .map(|header| format!("header\u{1f}{header}"));
        match cause_signature {
            Some(cause) => {
                if let Some(header) = header {
                    self.seen.insert(header);
                }
                !self.seen.insert(format!("cause\u{1f}{cause}"))
            }
            None => {
                let declined_header_seen = header
                    .as_ref()
                    .is_some_and(|header| self.seen.contains(header));
                !self.seen.insert(format!("text\u{1f}{}", text_signature())) || declined_header_seen
            }
        }
    }
}
