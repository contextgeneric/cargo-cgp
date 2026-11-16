//! The class detectors: recognizing a CGP mistake the compiler reports in its own terms, and
//! filling the rustc-free model the emitter words.
//!
//! The detectors import the resolver's [`query`](crate::resolve::query) layer and
//! [`cgp_item`](crate::resolve::query) recognizers, and one another, never the rest of the
//! resolver. See `cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md`.

mod ambiguous_names;
mod cgp_impl_misuse;
mod conflict;
mod macro_less_provider;
mod missing_use_provider;
mod orphan;
mod requirement;
mod structural;
mod undeclared;
mod unimported;
mod use_type_misspelling;
mod uses_misuse;
mod wiring_name;

pub use ambiguous_names::ambiguous_type_names;
pub use cgp_impl_misuse::{DetectedCgpImplMisuse, detect_cgp_impl_misuses};
pub use conflict::{ConflictAction, ConflictTrait, classify_wiring_conflict};
pub use macro_less_provider::detect_provider_impl_without_macro;
pub use missing_use_provider::detect_missing_use_provider;
pub use orphan::classify_orphan_conflict;
pub use requirement::{DetectedUndeclaredRequirement, detect_undeclared_requirement};
pub use structural::{
    DuplicateCheck, detect_check_at_unit_parameters, detect_duplicate_check,
    detect_duplicate_check_trait, detect_entry_parameters_not_in_key, detect_generic_default_impl,
    detect_loop_key_without_variable, is_from_auto_dispatch,
};
pub use undeclared::{UndeclaredCall, detect_undeclared_trait};
pub use unimported::{DetectedUnimportedAbstractType, detect_unimported_abstract_type};
pub use use_type_misspelling::detect_use_type_misspelling;
pub use uses_misuse::detect_provider_trait_in_uses;
pub use wiring_name::{
    NameAction, NameFix, UnresolvedNameCandidate, UnresolvedNamesPlan, derived_component_fix,
    derived_component_name, detect_consumer_trait_as_key, plan_unresolved_names,
    provider_header_at,
};
