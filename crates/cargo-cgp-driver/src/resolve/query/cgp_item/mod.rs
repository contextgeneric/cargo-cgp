//! DefId-anchored recognition of the CGP traits and types the resolver walks.
//!
//! Every stage of the resolution is anchored by `DefId` to the CGP crate that defines the
//! trait or type it matches, so a same-named item from an unrelated crate can never drive a
//! replacement, the same discipline [`component_map`](crate::component_map) uses for
//! `IsProviderFor`. The recognizers are grouped by what they recognize: an item by name, a
//! component's traits, a blanket trait, a namespace lookup trait, CGP's own crates, a context,
//! and the type-level spines.

mod blanket_trait;
mod cgp_crates;
mod component;
mod context;
mod name_anchor;
mod namespace_trait;
mod spine;

pub use blanket_trait::*;
pub use cgp_crates::*;
pub use component::*;
pub use context::*;
pub use name_anchor::*;
pub use namespace_trait::*;
pub use spine::*;
