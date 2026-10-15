//! Classifying the terminal leaf a dependency chain bottoms out on.
//!
//! Once the [walk](crate::resolve::walk) reaches a terminal predicate, this module turns it into
//! the rustc-free [`Leaf`](cargo_cgp_error_processing::Leaf) the emitter words. [`leaf`] performs
//! the classification, [`reportable`] decides whether a terminal is a real root cause or a routing
//! dead-end to drop, [`field`] inspects the actual struct a `HasField` bound lands on (and its
//! `Deref` chain) so a genuinely missing field is told apart from one present but underived,
//! [`required_type`] reads the type a requirement gives a missing field, [`near_miss`] finds a
//! similar field or wiring key the program wrote, [`assoc_type`] reads the type an owner actually
//! supplies for any other projected associated type, and [`other_arguments`] names the argument a
//! provider does not implement its provider trait for.

mod assoc_type;
mod data;
mod field;
mod leaf;
mod near_miss;
mod other_arguments;
mod reportable;
mod required_type;

pub(crate) use assoc_type::*;
pub(crate) use data::*;
pub(crate) use field::*;
pub(crate) use leaf::*;
pub(crate) use near_miss::*;
pub(crate) use other_arguments::*;
pub(crate) use reportable::*;
pub(crate) use required_type::*;
