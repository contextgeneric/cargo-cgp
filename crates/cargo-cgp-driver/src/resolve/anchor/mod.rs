//! Recovering the starting obligation of a check failure.
//!
//! Eight entry points here recover the obligation differently, then feed the same
//! [walk](crate::resolve::walk): [`resolve_check_failure`] anchors on a `check_components!` entry
//! (by matching the failing diagnostic's caret to the check impl's `Self`-type span), and
//! [`resolve_provider_check`] on a `#[check_providers]` entry the same way, seeding the walk with
//! the provider obligation it asserts;
//! [`resolve_impl_site`] handles a wiring failure surfaced *inside a hand-written `impl Trait for
//! Context` block* (by recovering the exact failing obligation — with its concrete component
//! parameters — from the impl's CGP consumer supertrait); [`resolve_wrapper_chain`] handles the same
//! shape when the impl's `Self` is a *foreign* wrapper holding the context (by descending its
//! supertrait's ordinary `where`-clause hops to a CGP consumer on the context, the routing-glue
//! case); [`resolve_use_site`] handles a
//! consumer-method `E0599` (by recovering the context ADT from the diagnostic's spans and
//! re-checking the parameterless form of every component that context wires);
//! [`resolve_use_site_consumer`] anchors on the consumer trait the diagnostic names, which is what
//! reaches a namespace-joined context; [`resolve_use_site_blanket_trait`] does the same for a
//! `#[cgp_fn]` or `#[blanket_trait]` blanket trait required as a bound; and
//! [`resolve_forwarding_entry`] reads a `Key -> Table` wiring entry whose own forwarding bound fails,
//! seeding the consumer the entry wires. Four more anchors live in
//! [`call_site`](crate::resolve::call_site), which re-read the failing call expression itself and
//! are tried before the blanket-trait anchor.
//!
//! The anchors share two ingredient modules: [`seed`] builds the real obligation each anchor feeds
//! the walk (a consumer obligation, or the provider obligation a `#[check_providers]` entry
//! asserts), and [`spans`] finds the local items the diagnostic's spans land on.

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
pub(crate) use seed::*;
pub(crate) use spans::*;
pub use use_site::*;
pub use use_site_consumer::*;
pub use wrapper_chain::*;
