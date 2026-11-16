//! The span-independent signature that identifies one wiring failure across its re-reports.

use crate::diagnosis::resolved::Resolved;
use crate::diagnosis::wording::lead::root_cause_lead;

/// A span-independent signature identifying *the same wiring failure* across the several
/// diagnostics one mistake produces, built from the context, the failing consumer traits, and each
/// leaf's root-cause lead. It is textual because the de-duplication ledger compares it with
/// rendered-message keys.
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/driver/program.md`.
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
