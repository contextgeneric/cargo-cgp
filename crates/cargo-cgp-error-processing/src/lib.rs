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
    Cause, DiagKind, DiagnosisPlan, FieldIssue, Leaf, Resolved, WiringConflict, WiringKey,
    cause_note, cause_notes, cause_signature, consumer_header, dependency_leaf_code,
    dependency_tree_leaf, derive_help_messages, field_mismatch_header, missing_delegate_entry,
    plan_resolved, plan_wiring_conflict, root_cause_code, root_cause_lead, wiring_conflict_help,
};
pub use postprocess::{
    CGP_PREFIXES, context_has_hasfield_impls, postprocess_message, resugar_lists, resugar_path,
    resugar_symbol, rewrite_missing_fields, strip_cgp_prefixes, strip_module_paths,
};
pub use rewrite::{ComponentNameMap, ComponentTraitNames, rewrite_message};
pub use tree::{DependencyTree, merge_dependency_forest, render_dependency_tree};
