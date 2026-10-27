//! The rustc-free diagnosis model and the diagnostic plan built from it.
//!
//! The driver's typed resolver produces a [`Resolved`] — the recovered root cause(s) of a check
//! failure, in owned `String` form — and this module turns it into the text the emitter emits.
//! [`plan_resolved`] composes the wording into a whole [`DiagnosisPlan`] (a rewritten header
//! and the replacement help/note messages); the emitter only maps that plan onto rustc's
//! `DiagInner`. Keeping every piece here rustc-free is what makes the diagnosis-to-text layer
//! unit-testable without a `TyCtxt`.

mod cgp_impl_misuse;
mod coalesce;
mod dispatch_names;
mod graph;
mod group;
mod internal_error;
mod leaf;
mod macro_less_provider;
mod node;
mod orphan;
mod overflow;
mod plan;
mod requirement;
mod resolved;
mod similar;
mod structural;
mod undeclared;
mod unimported;
mod use_type_misspelling;
mod wiring;
mod wiring_name;
mod wording;

pub use cgp_impl_misuse::*;
pub use coalesce::*;
pub use dispatch_names::*;
pub use graph::*;
pub use group::*;
pub use internal_error::*;
pub use leaf::*;
pub use macro_less_provider::*;
pub use node::*;
pub use orphan::*;
pub use overflow::*;
pub use plan::*;
pub use requirement::*;
pub use resolved::*;
pub use similar::*;
pub use structural::*;
pub use undeclared::*;
pub use unimported::*;
pub use use_type_misspelling::*;
pub use wiring::*;
pub use wiring_name::*;
pub use wording::*;
