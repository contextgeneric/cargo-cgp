//! The diagnostics the emitter holds until its flush.

use cargo_cgp_error_processing::{PendingNote, Resolved};
use rustc_errors::{DiagInner, Sublevel};
use rustc_span::Span;

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
    /// A diagnostic emitted verbatim at its arrival position unless the flush finds that another
    /// diagnostic, wherever it sits in the buffer, already states its mistake (see
    /// [`settle_pending`]). Deciding at the flush rather than on arrival makes the decision the
    /// same whichever of the two arrived first.
    Pending(Box<PendingEntry>),
}

/// The payload of a [`BufEntry::Pending`].
pub struct PendingEntry {
    /// The diagnostic, as it would be emitted.
    pub diag: DiagInner,
    /// What would make it a restatement.
    pub kind: PendingKind,
}

/// Why a [`BufEntry::Pending`] may restate another diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingKind {
    /// A `?`-operator error, a cascade of any CGP failure recognized at a span overlapping its own.
    Cascade,
    /// A failure the resolver declined and the text rewrite reworded, restating any resolved
    /// failure that recorded one of these coded headlines.
    Declined {
        /// Its coded class headlines.
        headers: Vec<String>,
    },
}

/// `buffer` with each pending entry settled: dropped when it restates another diagnostic, and kept
/// as a plain entry at its position otherwise.
///
/// Postcondition: every entry other than a pending one is kept, in order; a
/// [`PendingKind::Cascade`] entry is dropped exactly when one of its primary spans overlaps one of
/// `cgp_spans`, and a [`PendingKind::Declined`] one exactly when `recorded_header` holds for one of
/// its headers; a kept pending entry becomes [`BufEntry::Plain`] at its position.
pub fn settle_pending(
    buffer: Vec<BufEntry>,
    cgp_spans: &[Span],
    recorded_header: impl Fn(&str) -> bool,
) -> Vec<BufEntry> {
    buffer
        .into_iter()
        .filter_map(|entry| {
            let BufEntry::Pending(pending) = entry else {
                return Some(entry);
            };
            let restates = match &pending.kind {
                PendingKind::Cascade => pending
                    .diag
                    .span
                    .primary_spans()
                    .iter()
                    .any(|span| cgp_spans.iter().any(|seen| seen.overlaps(*span))),
                PendingKind::Declined { headers } => {
                    headers.iter().any(|header| recorded_header(header))
                }
            };
            (!restates).then(|| BufEntry::Plain(Box::new(pending.diag)))
        })
        .collect()
}

/// The payload of a [`BufEntry::Resolved`].
pub struct ResolvedEntry {
    /// Whether the flush may merge this entry with others that share a root cause.
    pub coalescible: bool,
    /// The failure the walk recovered.
    pub resolved: Resolved,
    /// The `root cause:` note, rendered at flush.
    pub note: PendingNote,
    /// The transformed diagnostic.
    pub diag: DiagInner,
    /// The diagnostic as rustc built it, emitted instead of the transformed one when the flush
    /// cannot render the transformed one (see [`BufEntry::untransformed`]).
    pub original: DiagInner,
}

impl BufEntry {
    /// The diagnostic to emit when the flush cannot render the buffer: a plain or pending entry as
    /// it stands, which a crash keeps rather than settles,
    /// and a resolution as rustc built it, since its note was never rendered. `note`, when given,
    /// is attached to such a resolution to say why it is untransformed.
    pub fn untransformed(self, note: Option<&str>) -> DiagInner {
        match self {
            BufEntry::Plain(diag) => *diag,
            BufEntry::Pending(pending) => pending.diag,
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
