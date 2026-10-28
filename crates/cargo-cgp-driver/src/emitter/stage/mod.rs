//! The emitter's stages, one module each, in the order the emitter runs them.
//!
//! A stage with memory of its own is a struct that owns it; a stage without is a function. Each
//! takes only the values it reads and returns a [`Verdict`](crate::emitter::pipeline::Verdict).

mod cgp_impl_misuse;
mod consumer_trait_key;
mod orphan;
mod provider_trait_in_uses;
mod shared_dispatch_method;
mod unresolved_overflow;
mod wiring_conflict;

pub use cgp_impl_misuse::*;
pub use consumer_trait_key::*;
pub use orphan::*;
pub use provider_trait_in_uses::*;
pub use shared_dispatch_method::*;
pub use unresolved_overflow::*;
pub use wiring_conflict::*;
