//! The `cargo-cgp` compiler-free diagnostic helpers.
//!
//! This crate holds the string-level diagnostic logic the driver drives but keeps out of
//! its `rustc_private` linkage, so it builds and its tests run on any toolchain. The driver
//! (`cargo-cgp-driver`) is the only caller; the front-end no longer touches diagnostics.
//! See `docs/implementation/error-processing.md` for the design.
//!
//! Three tenants live here, all driven by the driver's emitter:
//!
//! - [`rewrite`] — the string transform that renames CGP wiring messages, over the
//!   [`ComponentNameMap`] the driver fills in from the compiler.
//! - [`postprocess`] — the fallback text transforms ([`postprocess_message`]) the driver
//!   applies to a diagnostic it did not rewrite, so raw CGP constructs stay readable.
//! - [`tree`] — the [`DependencyTree`] and its `cargo tree`-style renderer the driver's
//!   typed resolver uses to show a check failure's dependency chain.

pub mod code;
pub mod postprocess;
pub mod rewrite;
pub mod tree;

pub use postprocess::{
    CGP_PREFIXES, context_has_hasfield_impls, postprocess_message, resugar_path, resugar_symbol,
    rewrite_missing_fields, strip_cgp_prefixes,
};
pub use rewrite::{ComponentNameMap, ComponentTraitNames, rewrite_message};
pub use tree::{DependencyTree, render_dependency_tree};
