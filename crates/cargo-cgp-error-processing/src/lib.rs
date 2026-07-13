//! The `cargo-cgp` compiler-free diagnostic helpers.
//!
//! This crate holds the string-level diagnostic logic the driver drives but keeps out of
//! its `rustc_private` linkage, so it builds and its tests run on any toolchain. The driver
//! (`cargo-cgp-driver`) is the only caller; the front-end no longer touches diagnostics.
//! See `docs/implementation/error-processing.md` for the design.
//!
//! Four tenants live here, all driven by the driver's emitter:
//!
//! - [`rewrite`] — the string transform that renames CGP wiring messages, over the
//!   [`ComponentNameMap`] the driver fills in from the compiler.
//! - [`postprocess`] — the fallback text transforms ([`postprocess_message`]) the driver
//!   applies to a diagnostic it did not rewrite, so raw CGP constructs stay readable.
//! - [`diagnosis`] — the rustc-free root-cause model ([`Resolved`]) the driver's typed
//!   resolver produces, and [`plan_resolved`], which words it into the header, help, and note
//!   text the emitter emits.
//! - [`tree`] — the [`DependencyTree`] and its `cargo tree`-style renderer the diagnosis
//!   wording uses to show a check failure's dependency chain.
//!
//! A fifth module, [`code`], holds the `CGP-E` error-code constants the rewrite and the
//! diagnosis wording stamp on classified main messages.

pub mod code;
pub mod diagnosis;
pub mod postprocess;
pub mod rewrite;
pub mod tree;

pub use diagnosis::{
    Cause, DiagKind, DiagnosisPlan, FieldIssue, Leaf, Resolved, cause_note, consumer_header,
    derive_help_messages, field_mismatch_header, plan_resolved,
};
pub use postprocess::{
    CGP_PREFIXES, context_has_hasfield_impls, postprocess_message, resugar_path, resugar_symbol,
    rewrite_missing_fields, strip_cgp_prefixes,
};
pub use rewrite::{ComponentNameMap, ComponentTraitNames, rewrite_message};
pub use tree::{DependencyTree, render_dependency_tree};
