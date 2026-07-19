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
