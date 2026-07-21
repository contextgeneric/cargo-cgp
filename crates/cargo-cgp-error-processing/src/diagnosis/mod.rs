//! The rustc-free diagnosis model and the diagnostic plan built from it.
//!
//! The driver's typed resolver produces a [`Resolved`] — the recovered root cause(s) of a check
//! failure, in owned `String` form — and this module turns it into the text the emitter emits.
//! [`plan_resolved`] composes the [wording] into a whole [`DiagnosisPlan`] (a rewritten header
//! and the replacement help/note messages); the emitter only maps that plan onto rustc's
//! `DiagInner`. Keeping every piece here rustc-free is what makes the diagnosis-to-text layer
//! unit-testable without a `TyCtxt`.

mod coalesce;
mod labels;
mod leaf;
mod orphan;
mod plan;
mod resolved;
mod wiring;
mod wording;

pub use coalesce::*;
pub use labels::*;
pub use leaf::*;
pub use orphan::*;
pub use plan::*;
pub use resolved::*;
pub use wiring::*;
pub use wording::*;
