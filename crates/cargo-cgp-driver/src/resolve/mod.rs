//! Typed root-cause resolution for CGP check-trait failures.
//!
//! When the emitter sees a CGP wiring failure, it asks this module to recover the *real* root
//! cause(s) — and the transitive dependency chain that leads to each — by walking the wiring's
//! trait obligations rather than by reading the rendered error text. The result is the rustc-free
//! [`Resolved`](cargo_cgp_error_processing::Resolved) the emitter turns into a diagnostic through
//! [`plan_resolved`](cargo_cgp_error_processing::plan_resolved). See
//! `docs/implementation/typed-root-cause-resolution.md` for the design.
//!
//! The pipeline is split by stage: [`anchor`] recovers the starting obligation (from a check
//! entry, a hand-written impl on the context, a foreign wrapper whose `where`-clause chain reaches
//! the context, or a use site) with [`call_site`] as the last resort (re-reading the failing call
//! expression itself, the unknowable parameters seeded as placeholders), [`walk`] descends the
//! dependency graph to each terminal leaf, [`classify`]
//! turns a leaf into a [`Leaf`](cargo_cgp_error_processing::Leaf) by inspecting the struct it lands
//! on, [`label`] renders each path predicate as a tree label, and [`cgp_item`] holds the
//! DefId-anchored CGP-trait recognition every stage relies on.
//!
//! A separate [`conflict`] stage handles the duplicate-key coherence conflict (`E0119`) rather
//! than a check failure: it reads the two conflicting `DelegateComponent` impls off the compiler
//! and words which keys collide. A sibling [`orphan`] stage handles the orphan-rule coherence
//! conflict (`E0210`/`E0117`) the same way: it reads the offending namespace-registration impl off
//! the compiler and words which foreign namespace and key it names.

mod anchor;
mod cache;
mod call_site;
mod cgp_impl_misuse;
mod cgp_item;
mod classify;
mod conflict;
mod label;
mod orphan;
mod undeclared;
mod walk;

pub use anchor::{
    resolve_check_failure, resolve_impl_site, resolve_use_site, resolve_use_site_capability,
    resolve_use_site_consumer, resolve_wrapper_chain,
};
pub use cache::ResolveCache;
pub use call_site::resolve_call_site;
pub use cgp_impl_misuse::{DetectedCgpImplMisuse, detect_cgp_impl_misuses};
pub use conflict::{ConflictAction, ConflictTrait, classify_wiring_conflict};
pub use orphan::classify_orphan_conflict;
pub use undeclared::detect_undeclared_capability;
