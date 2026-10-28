//! The `DiagInner`-editing helpers the emitter drives.
//!
//! These are the small functions that read from or rewrite a `DiagInner` in place: the seam
//! between the rustc diagnostic type and the rustc-free [error
//! processing](cargo_cgp_error_processing) that decides *what* the text should say. Keeping them
//! here leaves [`CgpEmitter`](super::CgpEmitter) with only the orchestration.

mod children;
mod readers;
mod spans;
mod text;

pub use children::*;
pub use readers::*;
pub use spans::*;
pub use text::*;
