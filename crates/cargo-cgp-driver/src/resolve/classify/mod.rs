//! Classifying the terminal leaf a dependency chain bottoms out on.
//!
//! Once the [walk](crate::resolve::walk) reaches a terminal predicate, this module turns it into
//! the rustc-free [`Leaf`](cargo_cgp_error_processing::Leaf) the emitter words. [`leaf`] performs
//! the classification, [`reportable`] decides whether a terminal is a real root cause or a routing
//! dead-end to drop, and [`field`] inspects the actual struct a `HasField` bound lands on (and its
//! `Deref` chain) so a genuinely missing field is told apart from one present but underived.

mod field;
mod leaf;
mod reportable;

pub(crate) use field::*;
pub(crate) use leaf::*;
pub(crate) use reportable::*;
