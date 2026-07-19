//! Recovering the starting obligation of a check failure.
//!
//! Five entry points recover the obligation differently, then feed the same
//! [walk](crate::resolve::walk): [`resolve_check_failure`] anchors on a `check_components!` entry
//! (by matching the failing diagnostic's caret to the check impl's `Self`-type span);
//! [`resolve_impl_site`] handles a wiring failure surfaced *inside a hand-written `impl Trait for
//! Context` block* (by recovering the exact failing obligation — with its concrete component
//! parameters — from the impl's CGP consumer supertrait); [`resolve_wrapper_chain`] handles the same
//! shape when the impl's `Self` is a *foreign* wrapper holding the context (by descending its
//! supertrait's ordinary `where`-clause hops to a CGP consumer on the context, the routing-glue
//! case); [`resolve_use_site`] handles a
//! consumer-method `E0599` (by recovering the context ADT from the diagnostic's spans and
//! re-checking the parameterless form of every component that context wires); and
//! [`resolve_use_site_consumer`] anchors on the consumer trait the diagnostic names, which is what
//! reaches a namespace-joined context. A sixth anchor lives in its own module,
//! [`call_site`](crate::resolve::call_site): the last resort that re-reads the failing call
//! expression itself.
//!
//! The anchors share two ingredient modules: [`seed`] builds the real consumer obligation each
//! anchor feeds the walk, and [`spans`] finds the local items the diagnostic's spans land on.

mod check_failure;
mod impl_site;
mod seed;
mod spans;
mod use_site;
mod use_site_consumer;
mod wrapper_chain;

pub use check_failure::*;
pub use impl_site::*;
pub(crate) use seed::*;
pub(crate) use spans::*;
pub use use_site::*;
pub use use_site_consumer::*;
pub use wrapper_chain::*;
