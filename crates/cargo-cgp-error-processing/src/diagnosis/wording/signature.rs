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
///
/// This one is deliberately *textual*, unlike the structural key the emitter's coalescing groups on
/// ([`group_by_shared_cause`](crate::group_by_shared_cause)): the de-duplication ledger compares it
/// against the rendered-message keys it falls back to for a diagnostic the resolver declined, so
/// every key it holds has to be a string.
pub fn cause_signature(resolved: &Resolved) -> String {
    let mut consumers = resolved.consumers.clone();
    consumers.sort();
    let mut leads: Vec<String> = resolved
        .causes
        .iter()
        .map(|cause| root_cause_lead(&cause.leaf))
        .collect();
    leads.sort();
    // `\u{1f}` (unit separator) and `\u{1e}` (record separator) cannot occur in a type or trait
    // name, so joining on them makes the signature unambiguous without escaping.
    format!(
        "{}\u{1f}{}\u{1f}{}",
        consumers.join("\u{1e}"),
        resolved.context,
        leads.join("\u{1e}")
    )
}
