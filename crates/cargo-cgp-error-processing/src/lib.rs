//! The `cargo-cgp` compiler-free diagnostic helpers.
//!
//! This crate holds the string-level diagnostic logic the driver drives but keeps out of
//! its `rustc_private` linkage, so it builds and its tests run on any toolchain. The driver
//! (`cargo-cgp-driver`) is the only caller; the front-end no longer touches diagnostics.
//! See `docs/implementation/error-processing.md` for the design.
//!
//! Six tenants live here, all driven by the driver's emitter:
//!
//! - [`rewrite`] — the string transform that renames CGP wiring messages, over the
//!   [`ComponentNameMap`] the driver fills in from the compiler.
//! - [`postprocess`] — the fallback text transforms ([`postprocess_message`]) the driver
//!   applies to a diagnostic it did not rewrite, so raw CGP constructs stay readable.
//! - [`diagnosis`] — the rustc-free root-cause model ([`Resolved`]) the driver's typed
//!   resolver produces, [`plan_resolved`], which words it into the header, help, and note
//!   text the emitter emits, and the pure label constructors the driver's tree builder uses.
//! - [`tree`] — the [`DependencyTree`] and its `cargo tree`-style renderer the diagnosis
//!   wording uses to show a check failure's dependency chain.
//! - [`dedup`] — the [`DedupLedger`] that suppresses the re-reports one lazy-wiring mistake
//!   produces at many sites.
//! - [`signals`] — the stable rustc phrasings the emitter's candidate checks key on.
//!
//! A further module, [`code`], holds the `CGP-E` error-code constants the rewrite and the
//! diagnosis wording stamp on classified main messages.

pub mod code;
pub mod dedup;
pub mod diagnosis;
pub mod postprocess;
pub mod rewrite;
pub mod signals;
pub mod tree;

pub use dedup::DedupLedger;
pub use diagnosis::{
    Cause, CgpImplMisuse, ChainNode, DepNode, DependencyGraph, DiagKind, DiagnosisPlan, FieldIssue,
    Leaf, MissingUseProvider, OrphanConflict, OrphanTrigger, PendingNote, Resolved,
    UndeclaredCapability, WiringConflict, WiringKey, assoc_mismatch_header,
    assoc_mismatch_help_messages, cause_note, cause_notes, cause_only_signature, cause_signature,
    cgp_impl_misuse_help, coalesce_underived_fields, consumer_header, dependency_leaf_code,
    dependency_tree_leaf, derive_help_messages, field_mismatch_header, fix_help_messages,
    merge_causes_by_leaf, missing_delegate_entry, missing_use_provider_help, orphan_conflict_help,
    plan_cgp_impl_misuse, plan_missing_use_provider, plan_orphan_conflict, plan_resolved,
    plan_undeclared_capability, plan_wiring_conflict, root_cause_code, root_cause_lead,
    undeclared_capability_help, wiring_conflict_help,
};
pub use postprocess::{
    CGP_PREFIXES, context_has_hasfield_impls, postprocess_message, resugar_lists, resugar_path,
    resugar_symbol, rewrite_missing_fields, strip_cgp_prefixes, strip_module_paths,
};
pub use rewrite::{ComponentNameMap, ComponentTraitNames, rewrite_message};
pub use signals::{
    is_method_bounds_text, is_method_probe_advice_text, is_question_mark_cascade_text,
    is_unbounded_type_param_item_text, mentions_orphan_param_text, mentions_wiring_text,
};
pub use tree::{DependencyTree, render_dependency_tree};
