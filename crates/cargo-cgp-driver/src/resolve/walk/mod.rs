//! Walking the wiring's dependency graph down to each terminal root cause.
//!
//! From a starting obligation (recovered by [anchor](crate::resolve::anchor)) this descends the
//! failing trait obligations — following only the CGP wiring vocabulary and obligations on the
//! context itself — and collects every root→leaf path that bottoms out on a terminal unmet bound,
//! folding each into a [`Cause`](cargo_cgp_error_processing::Cause) with its rendered dependency
//! tree.
//!
//! The descent is split by concern: [`leaves`] drives the recursion, [`vocabulary`] decides which
//! obligations it walks into, [`impl_match`] finds the impl that satisfies an obligation and reads
//! its `where`-clause dependencies, [`projection_mismatch`] surfaces a field-type mismatch the
//! trait-clause walk cannot see, [`unknowns`] carries a call-site unknown across inference-context
//! boundaries as a rigid placeholder, and [`holds`] asks the solver whether a predicate is
//! satisfied.

mod holds;
mod impl_match;
mod leaves;
mod projection_mismatch;
mod unknowns;
mod vocabulary;

pub(crate) use holds::*;
pub(crate) use impl_match::*;
pub(crate) use leaves::*;
pub(crate) use projection_mismatch::*;
pub(crate) use unknowns::*;
pub(crate) use vocabulary::*;
