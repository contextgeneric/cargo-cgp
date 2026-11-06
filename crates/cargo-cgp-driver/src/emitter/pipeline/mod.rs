//! The stage pipeline the emitter runs each diagnostic through.
//!
//! Each stage reads the diagnostic and returns a [`Verdict`]: leave it to the next stage, drop it
//! as a consequence of a failure already reported, or reshape it. A [`Reshape`] holds every edit a
//! stage makes as data (the class's wording, the span to keep, and an edit of the diagnostics
//! already buffered), so [`apply_reshape`] and the emitter's tail write every reshape the same way.

mod apply;
mod verdict;

pub use apply::*;
pub use verdict::*;
