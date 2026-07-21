//! The span-independent signature that identifies one wiring failure across its re-reports.

use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::lead::root_cause_lead;

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
    // `\u{1f}` (unit separator) cannot occur in a type or trait name, so joining on it makes the
    // signature unambiguous without escaping.
    format!(
        "{}\u{1f}{}",
        consumers.join("\u{1e}"),
        cause_only_signature(resolved)
    )
}

/// A span- *and* consumer-independent signature identifying *the same root cause* across the
/// several consumers one mistake breaks. It is [`cause_signature`] with the failing consumer
/// trait(s) dropped, so two *different* consumers that bottom out on the same cause — a missing
/// field several components read, a dependency chain many providers share — carry an equal
/// signature. The emitter groups buffered failures by this key and coalesces each group into one
/// headline that lists every affected consumer, rather than emitting one block per consumer.
pub fn cause_only_signature(resolved: &Resolved) -> String {
    let mut leads: Vec<String> = resolved
        .causes
        .iter()
        .map(|cause| root_cause_lead(&cause.leaf))
        .collect();
    leads.sort();
    format!("{}\u{1f}{}", resolved.context, leads.join("\u{1e}"))
}
