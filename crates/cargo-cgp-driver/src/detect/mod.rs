//! The class detectors: recognizing a CGP mistake the compiler reports in its own terms.
//!
//! Each detector confirms one class of mistake from the compiler state at the error's caret and
//! fills the rustc-free model the emitter words, without the typed resolver's walk: `conflict`
//! reads two conflicting `DelegateComponent` impls (`E0119`) and words which keys collide, `orphan`
//! the namespace registration the orphan rule rejects (`E0210`/`E0117`), `structural` the mistakes
//! in a wiring or its checks (a check table listing an entry twice), `wiring_name` a wiring name
//! that does not resolve, and the lowering detectors (`undeclared`, `missing_use_provider`,
//! `unimported`, `requirement`, `cgp_impl_misuse`, `uses_misuse`, `macro_less_provider`,
//! `use_type_misspelling`) a dependency or a component half a CGP macro's input left out or
//! misnamed. `ambiguous_names` reads which type names the post-processing must leave qualified.
//!
//! The detectors import the resolver's [`query`](crate::resolve::query) layer and
//! [`cgp_item`](crate::resolve::query) recognizers, and one another, never the rest of the
//! resolver.

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
    detect_check_at_unit_parameters, detect_duplicate_check, detect_duplicate_check_trait,
    detect_entry_parameters_not_in_key, detect_generic_default_impl,
    detect_loop_key_without_variable, is_from_auto_dispatch,
};
pub use undeclared::detect_undeclared_trait;
pub use unimported::{DetectedUnimportedAbstractType, detect_unimported_abstract_type};
pub use use_type_misspelling::detect_use_type_misspelling;
pub use uses_misuse::detect_provider_trait_in_uses;
pub use wiring_name::{
    NameAction, UnresolvedNamesPlan, detect_consumer_trait_as_key, plan_unresolved_names,
};
