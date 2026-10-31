//! Descending the failing obligation to every terminal root-cause leaf.
//!
//! [`entry`] holds the walk's entry points and the fold of the root node's sub-result, [`node`]
//! resolves one node with the cycle guard, the depth cap, and the cache, [`children`] chooses what it
//! descends into, [`results`] builds the sub-result of a node that ends a path, [`context_wiring`]
//! reads the unmet lookups on the context's own wiring, and [`overflow_step`] names why a cut walk
//! never resolves.

mod children;
mod context_wiring;
mod entry;
mod node;
mod overflow_step;
mod results;

pub use children::*;
pub use context_wiring::*;
pub use entry::*;
pub use node::*;
pub use overflow_step::*;
pub use results::*;
