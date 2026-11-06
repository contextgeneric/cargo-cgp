//! Classifying the terminal leaf a dependency chain bottoms out on.
//!
//! Once the [walk](crate::resolve::walk) reaches a terminal predicate, this module turns it into
//! the rustc-free [`Leaf`](cargo_cgp_error_processing::Leaf) the emitter words. `leaf` performs the
//! classification, dropping a terminal that is a routing dead-end rather than a root cause, `data`
//! reads an extensible-data terminal off its presence markers, `reportable` holds the shapes that
//! tell a delegation table's missing entry from such a dead-end, `field` inspects the actual struct
//! a `HasField` bound lands on (and its `Deref` chain) so a genuinely missing field is told apart
//! from one present but underived, and `known` says whether a provider obligation is known well
//! enough to confirm a fix against. The fix facts a leaf's `help` names are read in `fix`. The
//! compiler queries these share with the walk and the detectors, such as the type an owner supplies
//! for a projected associated type, live in the [query layer](crate::resolve::query).

mod data;
mod field;
mod fix;
mod known;
mod leaf;
mod reportable;

pub use data::*;
pub use field::*;
pub use fix::*;
pub use known::*;
pub use leaf::*;
pub use reportable::*;
