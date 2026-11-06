//! The stage for an overflow that follows from a wiring value that does not resolve.

use rustc_errors::DiagInner;

use crate::emitter::name_resolution::is_unresolved_wiring_overflow;
use crate::emitter::pipeline::Verdict;

/// Drop an `E0275` overflow on a requirement that a wiring entry with an unresolved value left
/// unprovable (`unprovable`, which the name-resolution pass recorded in the output), since the
/// entry's own error, already reported at the value, explains it.
pub fn unresolved_wiring_overflow(diag: &DiagInner, unprovable: &[String]) -> Verdict {
    if is_unresolved_wiring_overflow(diag, unprovable) {
        Verdict::Drop
    } else {
        Verdict::Continue
    }
}
