//! Wording a resolved check failure as diagnostic text.
//!
//! These are the pure `Resolved`-to-`String` builders the emitter's plan is composed from:
//! [`header`] words the coded main messages, [`lead`] the one-sentence root-cause statements and
//! their codes, [`note`] the `root cause:` note bodies over their dependency chains, [`help`] the
//! `#[derive(HasField)]` fixes, and [`signature`] the span-independent key that identifies one
//! wiring failure across its many re-reports. Each is a plain function over the rustc-free
//! [`Resolved`](super::Resolved) model, so it is unit-tested without a compiler; the
//! [plan](super::plan) module composes them into the whole [`DiagnosisPlan`](super::DiagnosisPlan).

mod header;
mod help;
mod lead;
mod note;
mod signature;

pub use header::*;
pub use help::*;
pub use lead::*;
pub use note::*;
pub use signature::*;
