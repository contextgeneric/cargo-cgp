//! The flush: rendering the buffered diagnostics once every diagnostic has arrived.
//!
//! The emitter holds every diagnostic until its `Drop`, the only point after the compilation's
//! last diagnostic, so the flush can coalesce the failures that share a root cause into one block,
//! render each resolution's `root cause:` note against what earlier blocks drew, merge a diagnostic
//! rustc reported twice, and make the "detailed explanations" footer name only the codes still
//! shown.

mod footers;
mod merge;
mod note;
mod render;

use std::collections::HashSet;

use cargo_cgp_error_processing::ChainNode;
pub use footers::*;
pub use merge::*;
pub use note::*;
pub use render::*;

/// The subtrees a flush has already drawn, shared across its blocks so a later one `(*)`-elides
/// what an earlier one showed rather than repeating it.
pub type ChainNodeSet = HashSet<ChainNode>;
