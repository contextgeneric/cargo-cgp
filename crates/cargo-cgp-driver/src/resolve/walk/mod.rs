//! Walking the wiring's dependency graph down to each terminal root cause.
//!
//! From a starting obligation (recovered by [anchor](crate::resolve::anchor)) this descends the
//! failing trait obligations, following only the CGP wiring vocabulary and obligations on the
//! context itself, and collects every root→leaf path that bottoms out on a terminal unmet bound,
//! folding each into a [`Cause`](cargo_cgp_error_processing::Cause) with its rendered dependency
//! tree.
//!
//! The descent is split by concern: `leaves` drives the recursion, and `vocabulary` decides which
//! obligations it walks into. The compiler queries it asks (the impl that satisfies an obligation
//! and its `where`-clause dependencies, a field-type mismatch the trait-clause walk cannot see,
//! whether a predicate holds, the placeholders that carry a call-site unknown across inference
//! contexts, and the requirements of a provider impl whose `#[cgp_impl]` header names the consumer
//! trait) live in the [query layer](crate::resolve::query), which the classifiers share.

mod leaves;
mod vocabulary;

pub use leaves::*;
pub use vocabulary::*;
