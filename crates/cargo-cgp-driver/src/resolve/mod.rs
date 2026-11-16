//! Typed root-cause resolution for CGP check-trait failures, into the rustc-free
//! [`Resolved`](cargo_cgp_error_processing::Resolved) the emitter words through
//! [`plan_resolved`](cargo_cgp_error_processing::plan_resolved). See
//! `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/overview.md`.
//!
//! A resolver module imports only modules earlier in this order: `query`, `cache`, and `ctx`;
//! `data`; `label`; `classify`; `walk`; `call_site`; `anchor`; `overflow`; and `anchors`, the
//! ordered list of anchors [`resolve`] tries. The class detectors in [`detect`](crate::detect)
//! share the `query` layer.

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
