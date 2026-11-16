//! Recovering a use-site failure's obligation from the call expression's own HIR, never
//! `tcx.typeck`, which cannot be forced from the emitter.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/call-site.md` for the failure
//! shape, each recovery step, and the anchors this module holds.

mod bound_call;
mod chain;
mod data_call;
mod find_call;
mod provider_call;
mod receiver;
mod seed;
mod unbounded;

pub use bound_call::*;
pub use chain::*;
pub use data_call::*;
pub use find_call::*;
pub use provider_call::*;
pub use receiver::*;
pub use seed::*;
pub use unbounded::*;
