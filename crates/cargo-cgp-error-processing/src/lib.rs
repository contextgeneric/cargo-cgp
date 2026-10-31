//! The `cargo-cgp` compiler-free diagnostic helpers.
//!
//! This crate holds the string-level diagnostic logic the driver drives but keeps out of
//! its `rustc_private` linkage, so it builds and its tests run on any toolchain. The driver
//! (`cargo-cgp-driver`) is the only caller; the front-end no longer touches diagnostics.
//! See `cgp-knowledge-base/cargo-cgp/implementation/error-processing.md` for the design.
//!
//! Eleven tenants live here, all driven by the driver's emitter:
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
//! - [`naming`] — the rule CGP macros follow for the names they invent.
//! - [`shape`] — the `Struct!`/`Enum!` record and variant forms a field list renders to, shared
//!   by the text post-processing and the driver's typed renderer.
//! - [`candidates`]: [`bound_candidate_list`], which cuts the implementor list the driver's
//!   `--verbose` flag un-elides back to the length rustc prints by default.
//! - [`byte_range`]: the anchors' span decisions over plain byte ranges, which the driver converts
//!   each `Span` to.
//! - [`syntax`]: telling a string literal from the code around it in rendered text.
//!
//! A further module, [`code`], holds the `CGP-E` error-code constants the rewrite and the
//! diagnosis wording stamp on classified main messages.

pub mod byte_range;
pub mod candidates;
pub mod code;
pub mod dedup;
pub mod diagnosis;
pub mod naming;
pub mod postprocess;
pub mod rewrite;
pub mod shape;
pub mod signals;
pub mod syntax;
pub mod tree;

pub use candidates::{RUSTC_CANDIDATE_LIMIT, bound_candidate_list};
pub use dedup::DedupLedger;
pub use diagnosis::{
    Adapter, AdapterFix, COALESCED_SITE_LABEL, Cause, Causes, CgpImplMisuse, ChainNode, CheckForm,
    CheckTraitName, ClassPlan, CoreProviderFix, CoreProviderHint, DepNode, DependencyGraph,
    DeriveFix, DiagKind, DiagnosisPlan, EntryFix, EntryTable, FIRST_CHECK_TABLE_LABEL,
    FIRST_DISPATCH_METHOD_LABEL, FieldFix, FieldIssue, Leaf, MethodFix, MissingUseProvider,
    NearMiss, OrphanConflict, OrphanTrigger, OverflowStep, ParameterUse, PartialKind, PartialValue,
    PendingNote, ProvidedComponent, ProviderImplWithoutMacro, ProviderTraitInUses, ProvidesFix,
    Requirement, Resolved, ResolvedClass, SECOND_CHECK_TABLE_LABEL, SECOND_DISPATCH_METHOD_LABEL,
    StructuralMistake, SuggestedCall, UndeclaredRequirement, UndeclaredTrait,
    UnimportedAbstractType, UnresolvedWiringName, UseTypeMisspelling, WiringConflict, WiringKey,
    WiringOverflow, cause_headline, cause_notes, cause_signature, cgp_impl_misuse_help, class_code,
    class_sentence, coalesce_underived_fields, coalesced_headline, consumer_header,
    dependency_leaf_code, dependency_tree_leaf, dispatch_computer_name, dispatch_helper_method,
    fix_help_messages, group_by_shared_cause, internal_error_note, merged_cause,
    missing_use_provider_help, missing_wiring, orphan_conflict_help, plan_cgp_impl_misuse,
    plan_missing_use_provider, plan_orphan_conflict, plan_provider_impl_without_macro,
    plan_provider_trait_in_uses, plan_resolved, plan_structural_mistake,
    plan_undeclared_requirement, plan_undeclared_trait, plan_unimported_abstract_type,
    plan_unresolved_wiring_name, plan_wiring_conflict, provider_impl_without_macro_help,
    provider_trait_in_uses_help, rewrite_wiring_overflow, root_cause_lead, similar_key,
    similar_name, structural_mistake_help, structural_mistake_note, undeclared_requirement_helps,
    undeclared_trait_help, unimported_abstract_type_help, unresolved_wiring_name_help,
    use_type_misspelling_help, wiring_conflict_help, wiring_overflow_help, wiring_overflow_reason,
};
pub use naming::{is_reserved_name, reserved_names};
pub use postprocess::{
    AmbiguousNames, Origin, context_has_hasfield_impls, postprocess_fragments, postprocess_message,
    resugar_lists, resugar_path, resugar_symbol, rewrite_missing_fields, scan_head_tail,
    strip_module_paths, strip_unambiguous_paths,
};
pub use rewrite::{ComponentNameMap, ComponentTraitNames, rewrite_message};
pub use shape::{ShapeTag, render_enum_shape, render_struct_shape, render_variant, shape_name};
pub use signals::{
    explain_footer_codes, invokes_macro, is_explain_footer_text, is_method_bounds_text,
    is_method_probe_advice_text, is_missing_type_parameter_help_text,
    is_question_mark_cascade_text, is_recursion_limit_help_text, is_unbounded_type_param_item_text,
    mentions_orphan_param_text, mentions_wiring_text,
};
pub use syntax::{code_chars, string_literal_end};
pub use tree::{DependencyTree, render_dependency_tree};
