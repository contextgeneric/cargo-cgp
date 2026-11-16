//! Recovering the starting obligation of a check failure: each anchor reads one failure shape and
//! feeds the same [walk](crate::resolve::walk), with four more in
//! [`call_site`](crate::resolve::call_site). `seed` builds the obligation each anchor feeds the
//! walk, and `spans` finds the local items the diagnostic's spans land on.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/anchors.md`.

mod check_failure;
mod forward_entry;
mod impl_site;
mod provider_check;
mod seed;
mod spans;
mod use_site;
mod use_site_consumer;
mod wrapper_chain;

pub use check_failure::*;
pub use forward_entry::*;
pub use impl_site::*;
pub use provider_check::*;
pub use seed::*;
pub use spans::*;
pub use use_site::*;
pub use use_site_consumer::*;
pub use wrapper_chain::*;
