//! The shape the after-expansion passes share over the buffered diagnostics.

use crate::emitter::buffer::BufEntry;
use crate::emitter::containment::contain;
use crate::emitter::output::with_registered_buffer;

/// Run one pass over the buffer of the output registered on this thread: collect its candidates
/// under the output's lock, run `detect` on them with the lock released and any panic contained,
/// since a query run while detecting can emit a diagnostic that takes the same lock, and then, under
/// the lock again, `apply` the detection to the buffer. A query run while detecting can only append
/// to the buffer, but `apply` runs only when `collect` still finds the same candidates, so a rewrite
/// never lands on the wrong entry. Nothing happens when no output is registered, no candidate is
/// found, or the detection panics, which leaves every diagnostic as rustc wrote it.
pub fn buffer_pass<C: PartialEq, D>(
    collect: impl Fn(&[BufEntry]) -> Vec<C>,
    detect: impl FnOnce(&[C]) -> D,
    apply: impl FnOnce(&mut Vec<BufEntry>, &[C], D),
) {
    let Some(candidates) = with_registered_buffer(|buffer| collect(buffer)) else {
        return;
    };
    if candidates.is_empty() {
        return;
    }
    let Ok(detected) = contain(|| detect(&candidates)) else {
        return;
    };
    with_registered_buffer(|buffer| {
        if collect(buffer) == candidates {
            apply(buffer, &candidates, detected);
        }
    });
}
