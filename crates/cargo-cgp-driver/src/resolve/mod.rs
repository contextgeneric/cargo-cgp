//! Typed root-cause resolution for CGP check-trait failures.
//!
//! When the emitter sees a CGP wiring failure, it asks this module to recover the *real* root
//! cause(s) and the transitive dependency chain that leads to each by walking the wiring's
//! trait obligations rather than by reading the rendered error text. The result is the rustc-free
//! [`Resolved`](cargo_cgp_error_processing::Resolved) the emitter turns into a diagnostic through
//! [`plan_resolved`](cargo_cgp_error_processing::plan_resolved). See
//! `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/overview.md` for the design.
//!
//! The pipeline is split by stage: `anchor` recovers the starting obligation (from a check
//! entry, a hand-written impl on the context, a foreign wrapper whose `where`-clause chain reaches
//! the context, or a use site), `call_site` recovers it by re-reading the failing call expression
//! itself (the unknowable parameters seeded as placeholders), `anchors` holds the order they are
//! tried in, `walk` descends the dependency graph to each terminal leaf, `classify`
//! turns a leaf into a [`Leaf`](cargo_cgp_error_processing::Leaf) by inspecting the struct it lands
//! on, and `label` renders each path predicate as a tree label. The `query` layer holds the
//! DefId-anchored CGP-trait recognition every stage relies on.
//!
//! The compiler readers and solver helpers more than one stage uses sit in the `query` layer, which
//! the class detectors in [`detect`](crate::detect) share. A resolver module imports only modules
//! earlier in this order: `query`, `cache`, and `ctx`; `data`; `label`; `classify`;
//! `walk`; `call_site`; `anchor`; `overflow`; and `anchors`, the ordered list of anchors [`resolve`]
//! tries.

pub mod anchor;
mod anchors;
pub mod cache;
pub mod call_site;
pub mod classify;
mod ctx;
pub mod data;
pub mod label;
mod overflow;
pub mod query;
pub mod walk;

pub use anchors::{ANCHORS, Anchor, AnchorInput, resolve};
pub use ctx::ResolveCtx;
pub use overflow::{OverflowWalk, overflow_walk};
