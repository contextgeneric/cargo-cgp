//! The `root cause:` note a plan leaves for the flush to render.

use std::collections::HashSet;

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::node::ChainNode;
use crate::diagnosis::resolved::Cause;
use crate::diagnosis::wording::cause_notes_seen;

/// The `root cause:` note as the inputs it renders from, rather than as text.
///
/// Rendering is deferred because a note elides against what *other* notes of the same compilation
/// already drew (see [`DependencyGraph::render_seen`](crate::DependencyGraph::render_seen)), and only
/// the emitter's flush knows the order they will appear in. Keeping the note in this form rather than
/// as a rendered string alongside is deliberate: two representations of one note would let the tests
/// pin one while the emitter shows the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingNote {
    /// The coalesced causes, one per distinct root cause.
    pub causes: Vec<Cause>,
    /// The leaf the header states, if any, whose lead is then redundant.
    pub header_leaf: Option<Leaf>,
}

impl PendingNote {
    /// Render the note against `seen`, the subtrees the compilation's earlier notes already drew. A
    /// caller with no such context — a unit test, or any single-diagnostic use — passes a fresh set.
    /// Yields one note, or none when there are no causes.
    pub fn render(&self, seen: &mut HashSet<ChainNode>) -> Vec<String> {
        cause_notes_seen(&self.causes, self.header_leaf.as_ref(), seen)
    }
}
