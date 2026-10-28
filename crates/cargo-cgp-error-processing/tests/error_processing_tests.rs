//! The unit tests of the rustc-free error-processing crate: the diagnosis model and
//! its plan, each reshape class's wording and its `ClassPlan`, the dependency graph, coalescing,
//! de-duplication, grouping, the post-processing transforms, the wiring rewrite, and the signals.
//!
//! One test target per crate, with a module per subject, keeps the crate to a single test binary.

pub mod error_processing;
