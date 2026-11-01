//! Turning a resolved wiring failure into a plan for the emitter to apply.
//!
//! [`plan_resolved`] is the rustc-free heart of the typed-resolution transform: given the
//! recovered [`Resolved`](crate::diagnosis::Resolved), the diagnostic's kind and main-message text, and the component-name
//! map, it decides the rewritten main message (when the diagnostic is an identified CGP class)
//! and the replacement sub-messages (the fix `help`s and the `root cause:` note).
//! The driver's emitter feeds it those inputs from the live `DiagInner` and turns the returned
//! [`DiagnosisPlan`] strings into `rustc` sub-diagnostics, so all the wording logic is here,
//! unit-tested without a compiler, and the emitter is left with only `DiagInner` manipulation.

mod categorized_header;
mod diag_kind;
mod diagnosis_plan;
mod pending_note;

pub use diag_kind::*;
pub use diagnosis_plan::*;
pub use pending_note::*;
