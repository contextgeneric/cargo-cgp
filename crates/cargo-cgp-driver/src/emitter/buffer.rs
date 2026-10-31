//! The diagnostics the emitter holds until its flush.

use cargo_cgp_error_processing::{PendingNote, Resolved};
use rustc_errors::{DiagInner, Sublevel};

use crate::emitter::edit::subdiag;

/// One buffered diagnostic awaiting the [`Drop`]-time flush.
pub enum BufEntry {
    /// Emitted verbatim at its arrival position. Boxed, like the variant below, so the buffer's
    /// element stays a pointer rather than a whole `DiagInner`.
    Plain(Box<DiagInner>),
    /// A typed-resolution failure, whose `root cause:` note is rendered at *flush* rather than on
    /// arrival: only there is the emission order known, so a note can `(*)`-elide the subtrees an
    /// earlier block already drew. `diag` therefore carries the header and `help`s but no note yet.
    ///
    /// `coalescible` is set for a failure whose headline gives way to its cause, which groups with
    /// every other failure it shares a root cause with (`group_by_shared_cause`): a group of one
    /// emits `diag` alone, a group of several emits a single merged block stating the shared cause
    /// at the position of the first. It is clear for a resolution that never coalesces: a type
    /// mismatch, or a kept rustc headline.
    Resolved(Box<ResolvedEntry>),
}

/// The payload of a [`BufEntry::Resolved`].
pub struct ResolvedEntry {
    pub coalescible: bool,
    pub resolved: Resolved,
    pub note: PendingNote,
    pub diag: DiagInner,
    /// The diagnostic as rustc built it, emitted instead of the transformed one when the flush
    /// cannot render the transformed one (see [`BufEntry::untransformed`]).
    pub original: DiagInner,
}

impl BufEntry {
    /// The diagnostic to emit when the flush cannot render the buffer: a plain entry as it stands,
    /// and a resolution as rustc built it, since its note was never rendered. `note`, when given,
    /// is attached to such a resolution to say why it is untransformed.
    pub fn untransformed(self, note: Option<&str>) -> DiagInner {
        match self {
            BufEntry::Plain(diag) => *diag,
            BufEntry::Resolved(entry) => {
                let mut diag = entry.original;
                if let Some(note) = note {
                    diag.children.push(subdiag(Sublevel::Note, note.to_owned()));
                }
                diag
            }
        }
    }
}
