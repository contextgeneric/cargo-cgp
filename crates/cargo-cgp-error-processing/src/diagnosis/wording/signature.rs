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
    let mut keys = cause_keys(resolved);
    keys.sort();
    // `\u{1f}` (unit separator) cannot occur in a type or trait name, so joining on it makes the
    // signature unambiguous without escaping.
    format!("{}\u{1f}{}", consumers.join("\u{1e}"), keys.join("\u{1e}"))
}

/// One span- *and* consumer-independent key per root cause of a failure, each scoped to the context
/// so two contexts never share one. A key identifies *one mistake* across every consumer it breaks:
/// two different consumers bottoming out on the same missing field, or on one dependency many
/// providers share, produce an equal key for it.
///
/// The emitter groups buffered failures on these keys ([`group_by_shared_cause`]) and coalesces each
/// group into one headline listing every affected consumer. Keys are returned per cause rather than
/// folded into one whole-failure signature deliberately: a failure reached at different depths sees
/// different *subsets* of one mistake's causes — a `check_components!` entry stops at the first unmet
/// leaf on its branch while a use-site call walks every wired component and reaches them all — so
/// grouping has to compare the causes one by one rather than demand two identical sets.
///
/// [`group_by_shared_cause`]: crate::group_by_shared_cause
pub fn cause_keys(resolved: &Resolved) -> Vec<String> {
    resolved
        .causes
        .iter()
        .map(|cause| format!("{}\u{1f}{}", resolved.context, root_cause_lead(&cause.leaf)))
        .collect()
}
